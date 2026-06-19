use blazing_fan_proto::{FrameError, UartRequest, UartResponse};
use thiserror::Error;

use crate::core::port::outbound::{fan_controller::FanControllerError, mcu::McuError};

#[derive(Error, Debug)]
pub enum UartError {
    #[error("emc2101 error {0}")]
    Emc(#[from] FanControllerError),
    #[error("rp2040 pico error {0}")]
    Rp(#[from] McuError),
    #[error("uart error")]
    Uart(embassy_rp::uart::Error),
    #[error("read exact error {0}")]
    ReadExact(embedded_io_async::ReadExactError<embassy_rp::uart::Error>),
    #[error("error serializing/deserealizing frame {0}")]
    Frame(#[from] FrameError),
}

impl From<embassy_rp::uart::Error> for UartError {
    fn from(value: embassy_rp::uart::Error) -> Self {
        Self::Uart(value)
    }
}

impl From<embedded_io_async::ReadExactError<embassy_rp::uart::Error>> for UartError {
    fn from(value: embedded_io_async::ReadExactError<embassy_rp::uart::Error>) -> Self {
        Self::ReadExact(value)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, defmt::Format)]
pub enum UartName {
    A,
    B,
}

pub trait UartPort {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError>;
}
