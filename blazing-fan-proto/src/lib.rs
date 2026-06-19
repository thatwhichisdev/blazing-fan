#![no_std]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize, defmt::Format, Default, Clone)]
pub struct FanTelemetry {
    pub fan_rpm: u16,
    pub fan_ctrl_temp_internal: i8,
    pub fan_ctrl_temp_external: i8,
    pub mcu_internal_temp: i8,
    pub mcu_system_voltage: u16,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize, defmt::Format)]
pub struct BladeTelemetry {
    pub cpu_temp: i8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize, defmt::Format)]
pub enum FanMode {
    Auto,
    Full,
    Idle,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize, defmt::Format)]
pub enum FanError {
    InvalidRequest,
    McuError,
    EmcError,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize, defmt::Format)]
pub enum UartRequest {
    Ping,
    Telemetry(BladeTelemetry),
    SetMode(FanMode),
}

#[derive(Serialize, Deserialize, Debug, MaxSize, defmt::Format)]
pub enum UartResponse {
    Pong,
    Ok,
    Error(FanError),
    Telemetry(FanTelemetry),
}

pub const UART_REQ_MAX_SIZE: usize = UartRequest::POSTCARD_MAX_SIZE;
pub const UART_RES_MAX_SIZE: usize = UartResponse::POSTCARD_MAX_SIZE;
