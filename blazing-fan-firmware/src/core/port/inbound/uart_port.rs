use blazing_fan_proto::{UartRequest, UartResponse};

#[derive(Debug, defmt::Format)]
pub enum UartError {}

pub trait UartPort {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError>;
}
