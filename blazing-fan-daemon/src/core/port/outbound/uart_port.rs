use blazing_fan_proto::{UartRequest, UartResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UartError {
    #[error("request timed out")]
    Timeout,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("postcard/serde error {0}")]
    PostcardError(#[from] postcard::Error),
}

pub trait UartPort {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError>;
}
