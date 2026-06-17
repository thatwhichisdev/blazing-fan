use blazing_fan_proto::{UartRequest, UartResponse};
use thiserror::Error;

use crate::core::port::outbound::{emc2101_port::Emc2101Error, rp2040_port::RP2040Error};

#[derive(Error, Debug, defmt::Format)]
pub enum UartError {
    #[error("emc2101 error")]
    EmcError(#[from] Emc2101Error),
    #[error("rp2040 pico error")]
    RpError(#[from] RP2040Error),
}

pub trait UartPort {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError>;
}
