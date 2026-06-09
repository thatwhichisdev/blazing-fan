#![no_main]
#![no_std]

mod pins;

use ariel_os::{
    gpio::{Input, Pull},
    hal,
};
use defmt::{error, info};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use postcard::experimental::max_size::{self, MaxSize};
use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};

use crate::pins::{ButtonPin, Peripherals, UartAPins, UartBPins};

use embedded_io_async::{Read as _, Write as _};

enum Mode {
    Manual,
    Auto,
    Idle,
}

struct OperatingSystem {
    mode: Mode,
    emc_tmp: i8,
    bld_a_tmp: i8,
    bld_b_tmp: i8,
}

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: Peripherals) {}

#[ariel_os::task(autostart, peripherals)]
async fn button_listener(peripherals: ButtonPin) {
    let mut button = Input::builder(peripherals.button, Pull::Up)
        .build_with_interrupt()
        .expect("BUTTON on PIN_12 should be present");

    loop {
        button.wait_for_any_edge().await;
        info!("Button state: {}", button.is_low());
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
enum UartCommand {
    Update { tmp: u8 },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
enum UartQuery {
    Fetch,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
enum UartRequest {
    Command(UartCommand),
    Query(UartQuery),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, MaxSize)]
enum UartResponse {
    Empty,
    Report { rpm: u8 },
}

const UART_REQ_MAX_SIZE: usize = UartRequest::POSTCARD_MAX_SIZE;
const UART_RES_MAX_SIZE: usize = UartResponse::POSTCARD_MAX_SIZE;

#[ariel_os::task(autostart, peripherals)]
async fn uart_a_listener(peripherals: UartAPins) {
    let uart_0_config = ariel_os::hal::uart::Config::default();
    let mut uart_0_rx_buf = [0u8; 32];
    let mut uart_0_tx_buf = [0u8; 32];
    let mut uart_a = pins::UartA::new(
        peripherals.uart0_rx,
        peripherals.uart0_tx,
        &mut uart_0_rx_buf,
        &mut uart_0_tx_buf,
        uart_0_config,
    )
    .expect("UART0 should be present");

    let mut read_buffer = [0u8; UART_REQ_MAX_SIZE];
    let mut write_buffer = [0u8; UART_RES_MAX_SIZE];

    loop {
        let Ok(()) = uart_a.read_exact(&mut read_buffer).await else {
            error!("Failed to read from uart_a");
            continue;
        };

        let request: UartRequest = from_bytes(&read_buffer).unwrap();
        let response: UartResponse = handle_uart_request(request).await;

        match response {
            UartResponse::Empty => continue,
            UartResponse::Report { .. } => {
                let data = to_slice(&response, &mut write_buffer).unwrap();
                uart_a.write(data).await.unwrap();
                uart_a.flush().await.unwrap();
            }
        }
    }
}

#[ariel_os::task(autostart, peripherals)]
async fn uart_b_listener(peripherals: UartBPins) {
    let uart_1_config = hal::uart::Config::default();
    let mut uart_1_rx_buf = [0u8; 32];
    let mut uart_1_tx_buf = [0u8; 32];
    let mut uart_b = pins::UartB::new(
        peripherals.uart1_rx,
        peripherals.uart1_tx,
        &mut uart_1_rx_buf,
        &mut uart_1_tx_buf,
        uart_1_config,
    )
    .expect("UART1 should be present");

    let mut read_buffer = [0u8; UART_REQ_MAX_SIZE];
    let mut write_buffer = [0u8; UART_RES_MAX_SIZE];

    loop {
        let Ok(()) = uart_b.read_exact(&mut read_buffer).await else {
            error!("Failed to read from uart_b");
            continue;
        };

        let request: UartRequest = from_bytes(&read_buffer).unwrap();
        let response: UartResponse = handle_uart_request(request).await;

        match response {
            UartResponse::Empty => continue,
            UartResponse::Report { .. } => {
                let data = to_slice(&response, &mut write_buffer).unwrap();
                uart_b.write(data).await.unwrap();
                uart_b.flush().await.unwrap();
            }
        }
    }
}

async fn handle_uart_request(request: UartRequest) -> UartResponse {
    match request {
        UartRequest::Command(UartCommand::Update { tmp }) => {
            info!("Received UartCommand::Update {{ tmp:{=u8} }}", tmp);
            UartResponse::Empty
        }
        UartRequest::Query(UartQuery::Fetch) => {
            info!("Received UartQuery::Fetch");
            UartResponse::Report { rpm: 0 }
        }
    }
}
