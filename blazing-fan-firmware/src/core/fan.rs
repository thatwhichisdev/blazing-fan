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
