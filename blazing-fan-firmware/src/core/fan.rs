use blazing_fan_proto::{UartCommand, UartQuery, UartRequest, UartResponse};
use bounded_integer::BoundedU8;
use smart_leds::RGB8;

use crate::ports::{
    button_port::ButtonPort,
    emc2101_port::Emc2101Port,
    fan_power_port::FanPowerPort,
    rp2040_port::RP2040Port,
    uart_port::{UartError, UartPort},
    ws2812_port::WS2812Port,
};

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
}

impl<B, E, P, W> Fan<B, E, P, W>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
    W: WS2812Port,
{
    pub fn new(brd: B, emc: E, pwr: P, pxl: W) -> Self {
        Self {
            brd,
            emc,
            pwr,
            pxl,
            mode: Mode::Auto,
        }
    }

    pub async fn tick(&mut self) {
        match self.mode {
            Mode::Auto => {
                self.pwr.pwr_on();
                self.brd.led_on();
                self.pxl.set_rgb8([RGB8::new(128, 0, 0); 2]).await;
                // todo: implement logic for automatically detecting fan speed
            }
            Mode::Full => {
                self.pwr.pwr_on();
                self.brd.led_on();
                self.pxl.set_rgb8([RGB8::new(0, 0, 128); 2]).await;
                self.emc
                    .set_fan_power(BoundedU8::<0, 63>::new(63).unwrap())
                    .await
                    .unwrap();
            }
            Mode::Idle => {
                self.pwr.pwr_off();
                self.brd.led_off();
                self.pxl.set_rgb8([RGB8::new(0, 0, 0); 2]).await;
                self.emc
                    .set_fan_power(BoundedU8::<0, 63>::new(0).unwrap())
                    .await
                    .unwrap();
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
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError> {
        match request {
            UartRequest::Command(command) => match command {
                UartCommand::Update { tmp } => {
                    defmt::debug!("CORE: Received command - Update [tmp: {=u8}]", tmp);

                    Ok(UartResponse::Ok)
                }
            },
            UartRequest::Query(query) => match query {
                UartQuery::FanGetStatus => {
                    defmt::debug!("CORE: Received query - FanGetStatus");

                    let fan_rpm = self.emc.fan_rpm().await.unwrap();
                    let fan_tmp_internal = self.emc.fan_tmp_internal().await.unwrap();
                    let fan_tmp_external = self.emc.fan_tmp_external().await.unwrap();
                    let brd_tmp = self.brd.board_tmp().unwrap();
                    let brd_vol = self.brd.board_sys_voltage().unwrap();

                    let uart_response = UartResponse::Status {
                        fan_rpm,
                        fan_tmp_internal,
                        fan_tmp_external,
                        brd_tmp,
                        brd_vol,
                    };

                    defmt::info!("CORE: Status [{:?}]", defmt::Debug2Format(&uart_response));

                    Ok(uart_response)
                }
            },
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
        self.mode = match self.mode {
            Mode::Auto => Mode::Full,
            Mode::Full => Mode::Idle,
            Mode::Idle => Mode::Auto,
        };
    }
}
