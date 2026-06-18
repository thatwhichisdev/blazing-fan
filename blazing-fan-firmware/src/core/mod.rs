pub mod port;

use crate::{
    adapter::inbound::uart_adapter::UartName,
    core::port::{
        inbound::{
            button_port::ButtonPort,
            uart_port::{UartError, UartPort},
        },
        outbound::{
            emc2101_port::{Emc2101Error, Emc2101Port},
            fan_power_port::FanPowerPort,
            rp2040_port::RP2040Port,
            ws2812_port::WS2812Port,
        },
    },
};

use blazing_fan_proto::{FanMode, FanTelemetry, UartRequest, UartResponse};
use bounded_integer::BoundedU8;
use core::cmp::max;
use heapless::index_map::FnvIndexMap;
use smart_leds::RGB8;
use thiserror::Error;

#[derive(Error, Debug, defmt::Format)]
pub enum FanError {
    #[error("emc error")]
    EmcError(#[from] Emc2101Error),
}

enum Mode {
    Auto,
    Full,
    Idle,
}

pub struct Fan<B, E, P, W>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
    W: WS2812Port,
{
    brd: B,
    emc: E,
    pwr: P,
    pxl: W,
    mode: Mode,
    tmps: FnvIndexMap<UartName, i8, 2>,
}

impl<B, E, P, W> Fan<B, E, P, W>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
    W: WS2812Port,
{
    pub fn new(brd: B, emc: E, pwr: P, pxl: W) -> Self {
        let mut tmps = FnvIndexMap::<_, _, 2>::new();
        tmps.insert(UartName::A, 0).expect("first entry");
        tmps.insert(UartName::B, 0).expect("second entry");

        Self {
            brd,
            emc,
            pwr,
            pxl,
            mode: Mode::Auto,
            tmps,
        }
    }

    fn set_mode(&mut self, mode: FanMode) {
        self.mode = match mode {
            FanMode::Auto => Mode::Auto,
            FanMode::Full => Mode::Full,
            FanMode::Idle => Mode::Idle,
        }
    }

    fn next_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Auto => Mode::Full,
            Mode::Full => Mode::Idle,
            Mode::Idle => Mode::Auto,
        };
    }

    pub async fn tick(&mut self) -> Result<(), FanError> {
        match self.mode {
            Mode::Auto => {
                self.pwr.pwr_on();
                self.brd.led_on();
                self.pxl.set_rgb8([RGB8::new(128, 0, 0); 2]).await;

                let bld_a_tmp = self
                    .tmps
                    .get(&UartName::A)
                    .expect("key is guaranteed to present by constructor of fan");

                let bld_b_tmp = self
                    .tmps
                    .get(&UartName::B)
                    .expect("key is guaranteed to present by constructor of fan");

                let tmp_max = max(bld_a_tmp, bld_b_tmp);

                let fan_pwr_u8 = match tmp_max {
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

                let fan_pwr = BoundedU8::<0, 63>::new(fan_pwr_u8)
                    .expect("provided value never exeeds the boundaries");

                self.emc.set_fan_power(fan_pwr).await?;

                Ok(())
            }
            Mode::Full => {
                self.pwr.pwr_on();
                self.brd.led_on();
                self.pxl.set_rgb8([RGB8::new(0, 0, 128); 2]).await;
                self.emc.set_fan_power(BoundedU8::<0, 63>::MAX).await?;

                Ok(())
            }
            Mode::Idle => {
                self.pwr.pwr_off();
                self.brd.led_off();
                self.pxl.set_rgb8([RGB8::new(0, 0, 0); 2]).await;
                self.emc.set_fan_power(BoundedU8::<0, 63>::MIN).await?;

                Ok(())
            }
        }
    }
}

impl<B, E, P, W> UartPort for Fan<B, E, P, W>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
    W: WS2812Port,
{
    async fn request(
        &mut self,
        request: UartRequest,
        port: &UartName,
    ) -> Result<UartResponse, UartError> {
        match request {
            UartRequest::Ping => Ok(UartResponse::Pong),
            UartRequest::SetMode(mode) => {
                self.set_mode(mode);

                Ok(UartResponse::Ok)
            }
            UartRequest::Telemetry(bld_telemetry) => {
                let bld_cpu_tmp = bld_telemetry.cpu_tmp;
                self.tmps
                    .insert(*port, bld_cpu_tmp)
                    .expect("key is always present");

                let fan_rpm = self.emc.fan_rpm().await?;
                let emc_tmp_internal = self.emc.fan_tmp_internal().await?;
                let emc_tmp_external = self.emc.fan_tmp_external().await?;
                let mcu_tmp = self.brd.mcu_tmp()?;
                let mcu_vol_mv = self.brd.mcu_sys_vol_mv()?;

                let telementry = FanTelemetry {
                    fan_rpm,
                    emc_tmp_internal,
                    emc_tmp_external,
                    mcu_tmp,
                    mcu_vol_mv,
                };

                Ok(UartResponse::Telemetry(telementry))
            }
        }
    }
}

impl<B, E, P, W> ButtonPort for Fan<B, E, P, W>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
    W: WS2812Port,
{
    async fn btn_pressed(&mut self) {
        defmt::info!("CORE: Changing mode");
        self.next_mode();
    }
}
