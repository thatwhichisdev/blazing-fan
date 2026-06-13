use blazing_fan_proto::{UartCommand, UartQuery, UartRequest, UartResponse};

use crate::ports::{
    button_port::ButtonPort,
    emc2101_port::Emc2101Port,
    fan_power_port::FanPowerPort,
    rp2040_port::RP2040Port,
    uart_port::{UartError, UartPort},
};

pub struct Fan<B, E, P>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
{
    brd: B,
    emc: E,
    pwr: P,
}

impl<B, E, P> Fan<B, E, P>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
{
    pub fn new(brd: B, emc: E, pwr: P) -> Self {
        Self { brd, emc, pwr }
    }

    pub async fn start(&mut self) {
        self.pwr.pwr_on();
        self.brd.led_on();
        self.brd.board_tmp().unwrap();
        self.brd.board_sys_voltage().unwrap();
    }
}

impl<B, E, P> UartPort for Fan<B, E, P>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
{
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError> {
        match request {
            UartRequest::Command(command) => match command {
                UartCommand::Update { tmp } => {
                    defmt::info!("CORE: Received command - Update [tmp: {=u8}]", tmp);

                    Ok(UartResponse::Empty)
                }
            },
            UartRequest::Query(query) => match query {
                UartQuery::FanGetRpm => {
                    defmt::info!("CORE: Received query - FanGetRpm");

                    let rpm = self.emc.fan_rpm().await.unwrap();

                    Ok(UartResponse::FanRpm { rpm })
                }
            },
        }
    }
}

impl<B, E, P> ButtonPort for Fan<B, E, P>
where
    B: RP2040Port,
    E: Emc2101Port,
    P: FanPowerPort,
{
    async fn btn_pressed(&mut self) {
        defmt::info!("CORE: Button pressed");
        // todo: to change state of the fan mode (looping thru modes)
    }
}
