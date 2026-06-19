#![no_std]

#[cfg(feature = "std")]
extern crate std;

use crc::{CRC_32_ISCSI, Crc};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
pub const PREAMBLE: u8 = 0xA5;
pub const CRC32_SIZE: usize = 4;
pub const REQUEST_MAX_SIZE: usize = UartRequest::POSTCARD_MAX_SIZE + CRC32_SIZE;
pub const RESPONSE_MAX_SIZE: usize = UartResponse::POSTCARD_MAX_SIZE + CRC32_SIZE;

#[derive(Error, Debug)]
#[cfg(feature = "no-std")]
pub enum FrameError {
    #[error("postcard error {0}")]
    Postcard(#[from] postcard::Error),
    #[error("preamble '{0}' doesn't match")]
    InvalidPreamble(u8),
    #[error("frame body length '{0}' exceeds request's max length")]
    ExceedsRequestMaxLength(u8),
    #[error("frame body length '{0}' exceeds response's max length")]
    ExceedsResponseMaxLength(u8),
    #[error("header length '{0}' doesn't match with body length '{0}'")]
    HeaderLengthMismatchedWithBodyLength(u8, u8),
    #[error("frame exceeds maximum size")]
    Capacity(heapless::CapacityError),
}

#[cfg(feature = "no-std")]
impl From<heapless::CapacityError> for FrameError {
    fn from(value: heapless::CapacityError) -> Self {
        FrameError::Capacity(value)
    }
}

#[derive(Error, Debug)]
#[cfg(feature = "std")]
pub enum FrameError {
    #[error("postcard error {0}")]
    Postcard(#[from] postcard::Error),
    #[error("preamble {0} doesn't match")]
    InvalidPreamble(u8),
    #[error("frame body length '{0}' exceeds request's max length")]
    ExceedsRequestMaxLength(u8),
    #[error("frame body length '{0}' exceeds response's max length")]
    ExceedsResponseMaxLength(u8),
    #[error("header length '{0}' doesn't match with body length '{0}'")]
    HeaderLengthMismatchedWithBodyLength(u8, u8),
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FrameHeader {
    pub preamble: u8,
    pub length: u8,
}

impl FrameHeader {
    pub fn as_slice(&self) -> [u8; 2] {
        [self.preamble, self.length]
    }
}

impl From<[u8; 2]> for FrameHeader {
    fn from(value: [u8; 2]) -> Self {
        Self {
            preamble: value[0],
            length: value[1],
        }
    }
}

#[cfg(feature = "std")]
#[derive(Debug, PartialEq, Eq)]
pub struct FrameBody<const N: usize> {
    pub data: std::vec::Vec<u8>,
}

#[cfg(feature = "no-std")]
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FrameBody<const N: usize> {
    pub data: heapless::Vec<u8, N>,
}

#[cfg(feature = "std")]
impl<const N: usize> FrameBody<N> {
    pub fn from_slice(data: &[u8]) -> Result<Self, FrameError> {
        Ok(Self {
            data: data.to_vec(),
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(feature = "no-std")]
impl<const N: usize> FrameBody<N> {
    pub fn from_slice(data: &[u8]) -> Result<Self, FrameError> {
        let mut body = heapless::Vec::<u8, N>::new();
        body.extend_from_slice(data)?;

        Ok(Self { data: body })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Frame<const N: usize> {
    pub header: FrameHeader,
    pub body: FrameBody<N>,
}

impl TryFrom<&UartRequest> for Frame<REQUEST_MAX_SIZE> {
    type Error = FrameError;

    fn try_from(value: &UartRequest) -> Result<Self, Self::Error> {
        Self::encode(value)
    }
}

impl TryFrom<&UartResponse> for Frame<RESPONSE_MAX_SIZE> {
    type Error = FrameError;

    fn try_from(value: &UartResponse) -> Result<Self, Self::Error> {
        Self::encode(value)
    }
}

impl TryFrom<&Frame<REQUEST_MAX_SIZE>> for UartRequest {
    type Error = FrameError;

    fn try_from(value: &Frame<REQUEST_MAX_SIZE>) -> Result<Self, Self::Error> {
        if value.header.preamble != PREAMBLE {
            return Err(FrameError::InvalidPreamble(value.header.preamble));
        }

        if value.header.length > REQUEST_MAX_SIZE as u8 {
            return Err(FrameError::ExceedsRequestMaxLength(value.header.length));
        }

        if value.header.length != value.body.len() as u8 {
            return Err(FrameError::HeaderLengthMismatchedWithBodyLength(
                value.header.length,
                value.body.len() as u8,
            ));
        }

        let decoded: UartRequest = postcard::from_bytes_crc32(&value.body.data, CRC32.digest())?;

        Ok(decoded)
    }
}

impl TryFrom<&Frame<RESPONSE_MAX_SIZE>> for UartResponse {
    type Error = FrameError;

    fn try_from(value: &Frame<RESPONSE_MAX_SIZE>) -> Result<Self, Self::Error> {
        if value.header.preamble != PREAMBLE {
            return Err(FrameError::InvalidPreamble(value.header.preamble));
        }

        if value.header.length > RESPONSE_MAX_SIZE as u8 {
            return Err(FrameError::ExceedsResponseMaxLength(value.header.length));
        }

        if value.header.length != value.body.len() as u8 {
            return Err(FrameError::HeaderLengthMismatchedWithBodyLength(
                value.header.length,
                value.body.len() as u8,
            ));
        }

        let decoded: UartResponse = postcard::from_bytes_crc32(&value.body.data, CRC32.digest())?;

        Ok(decoded)
    }
}

impl<const N: usize> Frame<N> {
    pub fn encode<T>(value: &T) -> Result<Self, FrameError>
    where
        T: Serialize,
    {
        let mut buffer = [0u8; N];
        let encoded = postcard::to_slice_crc32(value, &mut buffer, CRC32.digest())?;

        let body = FrameBody::from_slice(encoded)?;
        let header = FrameHeader {
            preamble: PREAMBLE,
            length: body.len() as u8,
        };

        Ok(Self { header, body })
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize, Default, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FanTelemetry {
    pub fan_rpm: u16,
    pub fan_ctrl_temp_internal: i8,
    pub fan_ctrl_temp_external: i8,
    pub mcu_internal_temp: i8,
    pub mcu_system_voltage: u16,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BladeTelemetry {
    pub cpu_temp: i8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FanMode {
    Auto,
    Full,
    Idle,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FanError {
    InvalidRequest,
    McuError,
    EmcError,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UartRequest {
    Ping,
    Telemetry(BladeTelemetry),
    SetMode(FanMode),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UartResponse {
    Pong,
    Ok,
    Err(FanError),
    Telemetry(FanTelemetry),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn create_frame_for_request() {
        let telemetry = BladeTelemetry { cpu_temp: 60 };
        let request = UartRequest::Telemetry(telemetry);
        let left = Frame::<REQUEST_MAX_SIZE>::try_from(&request);

        assert_eq!(left.is_ok(), true);

        let left = left.unwrap();
        let right: Frame<REQUEST_MAX_SIZE> = Frame {
            header: FrameHeader {
                preamble: 0xA5,
                length: 6,
            },
            body: FrameBody {
                data: std::vec![1, 60, 196, 105, 99, 159],
            },
        };

        assert_eq!(left, right);
    }

    #[test]
    #[cfg(feature = "std")]
    fn create_request_from_frame() {
        let frame: Frame<REQUEST_MAX_SIZE> = Frame {
            header: FrameHeader {
                preamble: 0xA5,
                length: 6,
            },
            body: FrameBody {
                data: std::vec![1, 60, 196, 105, 99, 159],
            },
        };
        let left = UartRequest::try_from(&frame);

        assert_eq!(left.is_ok(), true);

        let left = left.unwrap();
        let right = UartRequest::Telemetry(BladeTelemetry { cpu_temp: 60 });

        assert_eq!(left, right);
    }

    #[test]
    #[cfg(feature = "std")]
    fn create_frame_for_response() {
        let telemetry = FanTelemetry {
            fan_rpm: 5543,
            fan_ctrl_temp_internal: 32,
            fan_ctrl_temp_external: 33,
            mcu_internal_temp: 30,
            mcu_system_voltage: 3321,
        };
        let response = UartResponse::Telemetry(telemetry);
        let left = Frame::<RESPONSE_MAX_SIZE>::try_from(&response);

        assert_eq!(left.is_ok(), true);

        let left = left.unwrap();
        let right: Frame<RESPONSE_MAX_SIZE> = Frame {
            header: FrameHeader {
                preamble: 0xA5,
                length: 12,
            },
            body: FrameBody {
                data: std::vec![3, 167, 43, 32, 33, 30, 249, 25, 69, 26, 184, 121],
            },
        };

        assert_eq!(left, right);
    }

    #[test]
    #[cfg(feature = "std")]
    fn create_response_from_frame() {
        let frame: Frame<RESPONSE_MAX_SIZE> = Frame {
            header: FrameHeader {
                preamble: 0xA5,
                length: 12,
            },
            body: FrameBody {
                data: std::vec![3, 167, 43, 32, 33, 30, 249, 25, 69, 26, 184, 121],
            },
        };
        let left = UartResponse::try_from(&frame);

        assert_eq!(left.is_ok(), true);

        let left = left.unwrap();
        let right = UartResponse::Telemetry(FanTelemetry {
            fan_rpm: 5543,
            fan_ctrl_temp_internal: 32,
            fan_ctrl_temp_external: 33,
            mcu_internal_temp: 30,
            mcu_system_voltage: 3321,
        });

        assert_eq!(left, right);
    }
}
