#![no_std]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub struct FanTelemetry {
    pub fan_rpm: u16,
    pub emc_tmp_internal: i8,
    pub emc_tmp_external: i8,
    pub mcu_tmp: i8,
    pub mcu_vol_mv: u16,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub struct BladeTelemetry {
    pub cpu_tmp: i8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum FanMode {
    Auto,
    Full,
    Idle,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum FanError {
    InvalidRequest,
    McuError,
    EmcError,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartRequest {
    Ping,
    Telemetry(BladeTelemetry),
    SetMode(FanMode),
}

#[derive(Serialize, Deserialize, Debug, MaxSize)]
pub enum UartResponse {
    Pong,
    Ok,
    Error(FanError),
    Telemetry(FanTelemetry),
}

pub const UART_REQ_MAX_SIZE: usize = UartRequest::POSTCARD_MAX_SIZE;
pub const UART_RES_MAX_SIZE: usize = UartResponse::POSTCARD_MAX_SIZE;
