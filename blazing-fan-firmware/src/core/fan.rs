use blazing_fan_proto::{UartCommand, UartQuery, UartRequest, UartResponse};

use crate::ports::{
    button_port::ButtonPort,
    emc2101_port::Emc2101Port,
    fan_power_port::FanPowerPort,
    uart_port::{UartError, UartPort},
};

pub struct Fan<E, P>
where
    E: Emc2101Port,
    P: FanPowerPort,
{
    emc: E,
    pwr: P,
}

impl<E, P> Fan<E, P>
where
    E: Emc2101Port,
    P: FanPowerPort,
{
    pub fn new(emc: E, pwr: P) -> Self {
        Fan { emc, pwr }
    }

    pub async fn start(&mut self) {
        self.pwr.pwr_on();
    }
}

impl<E, P> UartPort for Fan<E, P>
where
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

impl<E, P> ButtonPort for Fan<E, P>
where
    E: Emc2101Port,
    P: FanPowerPort,
{
    async fn btn_pressed(&mut self) {
        defmt::info!("CORE: Button pressed");
        // todo: to change state of the fan mode (looping thru modes)
    }
}
