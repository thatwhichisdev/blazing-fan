#![no_std]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartCommand {
    Update { tmp: u8 },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartQuery {
    Fetch,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartRequest {
    Command(UartCommand),
    Query(UartQuery),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
pub enum UartResponse {
    Empty,
    Report { rpm: u16 },
}

pub const UART_REQ_MAX_SIZE: usize = UartRequest::POSTCARD_MAX_SIZE;
pub const UART_RES_MAX_SIZE: usize = UartResponse::POSTCARD_MAX_SIZE;
