use blazing_fan_proto::{UartRequest, UartResponse};
use thiserror::Error;

use crate::core::port::outbound::{fan_controller::FanControllerError, mcu::McuError};

#[derive(Error, Debug, defmt::Format)]
pub enum UartError {
    #[error("emc2101 error")]
    EmcError(#[from] FanControllerError),
    #[error("rp2040 pico error")]
    RpError(#[from] McuError),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, defmt::Format)]
pub enum UartName {
    A,
    B,
}

pub trait UartPort {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError>;
}
