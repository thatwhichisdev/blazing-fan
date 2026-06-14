use blazing_fan_proto::{UartRequest, UartResponse};

pub trait UartPort {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, ()>;
}
