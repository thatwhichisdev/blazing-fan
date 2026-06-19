pub mod port;

use crate::core::port::{
    inbound::{
        uart_port::{UartError, UartPort},
        user_button::UserButton,
    },
    outbound::{
        fan_controller::{FanController, FanControllerError},
        fan_supply::FanSupply,
        mcu::Mcu,
        status_indicator::StatusIndicator,
    },
};

use blazing_fan_proto::{FanMode, FanTelemetry, UartRequest, UartResponse};
use bounded_integer::BoundedU8;
use heapless::HistoryBuf;
use thiserror::Error;

#[derive(Error, Debug, defmt::Format)]
pub enum SystemError {
    #[error("fan controller error {0}")]
    FanControllerError(#[from] FanControllerError),
}

enum OperatingMode {
    Auto,
    Full,
    Idle,
}

pub struct System<M, C, S, I>
where
    M: Mcu,
    C: FanController,
    S: FanSupply,
    I: StatusIndicator,
{
    mcu: M,
    fan_ctrl: C,
    fan_supply: S,
    status_indicator: I,
    operating_mode: OperatingMode,
    telemetries: HistoryBuf<i8, 2>,
}

impl<M, C, S, I> System<M, C, S, I>
where
    M: Mcu,
    C: FanController,
    S: FanSupply,
    I: StatusIndicator,
{
    pub fn new(mcu: M, fan_ctrl: C, fan_supply: S, status_indicator: I) -> Self {
        let telemetries = HistoryBuf::new();

        Self {
            mcu,
            fan_ctrl,
            fan_supply,
            status_indicator,
            operating_mode: OperatingMode::Auto,
            telemetries,
        }
    }

    fn set_mode(&mut self, mode: FanMode) {
        self.operating_mode = match mode {
            FanMode::Auto => OperatingMode::Auto,
            FanMode::Full => OperatingMode::Full,
            FanMode::Idle => OperatingMode::Idle,
        }
    }

    fn next_mode(&mut self) {
        self.operating_mode = match self.operating_mode {
            OperatingMode::Auto => OperatingMode::Full,
            OperatingMode::Full => OperatingMode::Idle,
            OperatingMode::Idle => OperatingMode::Auto,
        };
    }

    pub async fn tick(&mut self) -> Result<(), SystemError> {
        match self.operating_mode {
            OperatingMode::Auto => {
                self.fan_supply.enable();
                self.mcu.led_on();
                self.status_indicator.set_green().await;

                if let Some(temp) = self.telemetries.iter().max() {
                    let fan_power_u8 = match temp {
                        i8::MIN..=44 => 0,
                        45..=49 => 13,
                        50..=54 => 19,
                        55..=59 => 25,
                        60..=64 => 32,
                        65..=69 => 41,
                        70..=74 => 50,
                        75..=79 => 57,
                        80..=i8::MAX => 63,
                    };

                    let fan_power = BoundedU8::<0, 63>::new(fan_power_u8)
                        .expect("fan_power_u8 value is not in the range from 0 to 63");

                    self.fan_ctrl.set_fan_power(fan_power).await?;
                    self.telemetries.clear();
                } else {
                    self.fan_ctrl.set_fan_auto().await?;
                }

                Ok(())
            }
            OperatingMode::Full => {
                self.fan_supply.enable();
                self.mcu.led_on();
                self.status_indicator.set_orange().await;
                self.fan_ctrl.set_fan_power_max().await?;

                Ok(())
            }
            OperatingMode::Idle => {
                self.fan_ctrl.set_fan_power_min().await?;
                self.fan_supply.disable();
                self.mcu.led_off();
                self.status_indicator.set_none().await;

                Ok(())
            }
        }
    }
}

impl<M, C, S, I> UartPort for System<M, C, S, I>
where
    M: Mcu,
    C: FanController,
    S: FanSupply,
    I: StatusIndicator,
{
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError> {
        match request {
            UartRequest::Ping => {
                defmt::info!("CORE: Received Ping request");

                Ok(UartResponse::Pong)
            }
            UartRequest::SetMode(mode) => {
                defmt::info!("CORE: Received SetMode request");

                self.set_mode(mode);

                Ok(UartResponse::Ok)
            }
            UartRequest::Telemetry(bld_telemetry) => {
                defmt::info!("CORE: Received Telemetry request {:?}", bld_telemetry);

                let blade_temp = bld_telemetry.cpu_temp;

                self.telemetries.write(blade_temp);

                let fan_rpm = self.fan_ctrl.get_fan_rpm().await?;
                let fan_ctrl_temp_internal = self.fan_ctrl.get_fan_tmp_internal().await?;
                let fan_ctrl_temp_external = self.fan_ctrl.get_fan_tmp_external().await?;
                let mcu_internal_temp = self.mcu.get_internal_temp()?;
                let mcu_system_voltage = self.mcu.get_system_voltage()?;

                let telementry = FanTelemetry {
                    fan_rpm,
                    fan_ctrl_temp_internal,
                    fan_ctrl_temp_external,
                    mcu_internal_temp,
                    mcu_system_voltage,
                };

                Ok(UartResponse::Telemetry(telementry))
            }
        }
    }
}

impl<M, C, S, I> UserButton for System<M, C, S, I>
where
    M: Mcu,
    C: FanController,
    S: FanSupply,
    I: StatusIndicator,
{
    async fn on_pressed(&mut self) {
        self.next_mode();
    }
}
