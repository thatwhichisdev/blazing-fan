#![no_std]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartCommand {
    Update { tmp: u8 },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartQuery {
    FanGetStatus,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartRequest {
    Command(UartCommand),
    Query(UartQuery),
}

#[derive(Serialize, Deserialize, Debug, MaxSize)]
pub enum UartResponse {
    Ok,
    Err,
    Status {
        fan_rpm: u16,
        fan_tmp_internal: i8,
        fan_tmp_external: i8,
        brd_tmp: i8,
        brd_vol: f32,
    },
}

pub const UART_REQ_MAX_SIZE: usize = UartRequest::POSTCARD_MAX_SIZE;
pub const UART_RES_MAX_SIZE: usize = UartResponse::POSTCARD_MAX_SIZE;
