#![no_main]
#![no_std]

mod pins;

use ariel_os::{
    gpio::{Input, Pull},
    hal,
};
use blazing_fan_proto::{
    UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE, UartCommand, UartQuery, UartRequest, UartResponse,
};
use defmt::{error, info};
use fugit::Rate;
use postcard::{from_bytes, to_slice};

use crate::pins::{ButtonPin, EmcI2C, Peripherals, UartAPins, UartBPins};

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
async fn main(peripherals: Peripherals) {
    let i2c_config = hal::i2c::controller::Config::default();
    let i2c0 = EmcI2C::new(peripherals.emc.sda, peripherals.emc.scl, i2c_config);
    let mut emc = emc2101::AsyncEMC2101::new(i2c0).await.unwrap();

    emc.enable_tach_input().await.unwrap();
    emc.set_fan_pwm(Rate::<u32, _, _>::kHz(25), false)
        .await
        .expect("should set fan pwm");

    let external_temp = emc.temp_external().await.unwrap();
    let internal_temp = emc.temp_internal().await.unwrap();

    info!("emc external temp {=i8}", external_temp);
    info!("emc internal temp {=i8}", internal_temp);
}

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

        info!("uart_b received data: {=[u8]}", read_buffer);

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
            UartResponse::Report { rpm: 4000 }
        }
    }
}
