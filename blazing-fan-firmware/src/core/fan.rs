use blazing_fan_proto::{UartCommand, UartQuery, UartRequest, UartResponse};

use crate::ports::{
    button_port::ButtonPort,
    emc2101_port::Emc2101Port,
    uart_port::{UartError, UartPort},
};

pub struct Fan<E>
where
    E: Emc2101Port,
{
    emc: E,
}

impl<E> Fan<E>
where
    E: Emc2101Port,
{
    pub fn new(emc: E) -> Self {
        Fan { emc }
    }
}

impl<E> UartPort for Fan<E>
where
    E: Emc2101Port,
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

impl<E> ButtonPort for Fan<E>
where
    E: Emc2101Port,
{
    async fn btn_pressed(&mut self) {
        defmt::info!("CORE: Button pressed");
        // todo: to change state of the fan mode (looping thru modes)
    }
}
