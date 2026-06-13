#![no_main]
#![no_std]

mod adapters;
mod core;
mod pins;
mod ports;

use crate::{
    adapters::{
        button_adapter::ButtonAdapter,
        emc2101_adapter::Emc2101Adapter,
        uart_adapter::{UartAdapter, UartName},
    },
    core::fan::Fan,
    pins::{ButtonPin, EmcPins, FanPowerPin, Peripherals, UartAPins, UartBPins},
};
use ariel_os::{
    asynch::Spawner,
    gpio::{Input, Level, Output},
    hal::{i2c, uart},
};
use blazing_fan_proto::{UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, watch::Watch};
use static_cell::StaticCell;

/// Fan should be wrapped in Mutex since we're going to share it across multiple tasks
static FAN: StaticCell<Mutex<CriticalSectionRawMutex, Fan<Emc2101Adapter>>> = StaticCell::new();
static FAN_READY: Watch<
    CriticalSectionRawMutex,
    &'static Mutex<CriticalSectionRawMutex, Fan<Emc2101Adapter>>,
    3,
> = Watch::new();

#[ariel_os::spawner(autostart, peripherals)]
fn boot(spawner: Spawner, peripherals: Peripherals) {
    spawner
        .spawn(core_task(peripherals.emc_pins, peripherals.fan_pwr_pin))
        .unwrap();

    spawner
        .spawn(button_adapter_task(peripherals.btn_pin))
        .unwrap();

    spawner
        .spawn(uart_a_adapter_task(peripherals.uart_a_pins))
        .unwrap();

    spawner
        .spawn(uart_b_adapter_task(peripherals.uart_b_pins))
        .unwrap();
}

#[ariel_os::task]
async fn core_task(emc_pins: EmcPins, fan_pwr_pin: FanPowerPin) {
    defmt::info!("Booting firmware");

    let _fan_power = Output::new(fan_pwr_pin.pwr, Level::High);

    let i2c_config = i2c::controller::Config::default();
    let i2c0 = i2c::controller::I2C0::new(emc_pins.sda, emc_pins.scl, i2c_config);

    let emc2101_adapter = Emc2101Adapter::new(i2c0).await;

    let fan: &'static _ = FAN.init(Mutex::new(Fan::new(emc2101_adapter)));

    let fan_signal_sender = FAN_READY.dyn_sender();
    fan_signal_sender.send(fan);
}

#[ariel_os::task]
async fn button_adapter_task(pins: ButtonPin) {
    defmt::info!("Booting external button");

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().unwrap();
    let fan = fan_signal_rcv.get().await;

    let btn = Input::builder(pins.button, ariel_os::gpio::Pull::Up)
        .build_with_interrupt()
        .unwrap();

    let mut button_adapter = ButtonAdapter::new(btn, fan);
    button_adapter.start().await;
}

#[ariel_os::task]
async fn uart_a_adapter_task(pins: UartAPins) {
    defmt::info!("Booting UART_A");

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().unwrap();
    let fan = fan_signal_rcv.get().await;

    let uart_config = uart::Config::default();
    let mut rx_buf = [0u8; UART_REQ_MAX_SIZE];
    let mut tx_buf = [0u8; UART_RES_MAX_SIZE];
    let uart = uart::UART0::new(
        pins.uart0_rx,
        pins.uart0_tx,
        &mut rx_buf,
        &mut tx_buf,
        uart_config,
    )
    .expect("UART0 should be present");

    let mut uart_a_adapter = UartAdapter::new(uart, &mut rx_buf, &mut tx_buf, fan, UartName::A);

    uart_a_adapter.start().await;
}

#[ariel_os::task]
async fn uart_b_adapter_task(pins: UartBPins) {
    defmt::info!("Booting UART_B");

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().unwrap();
    let fan = fan_signal_rcv.get().await;

    let uart_config = uart::Config::default();
    let mut rx_buf = [0u8; UART_REQ_MAX_SIZE];
    let mut tx_buf = [0u8; UART_RES_MAX_SIZE];
    let uart = uart::UART1::new(
        pins.uart1_rx,
        pins.uart1_tx,
        &mut rx_buf,
        &mut tx_buf,
        uart_config,
    )
    .expect("UART1 should be present");

    let mut uart_b_adapter = UartAdapter::new(uart, &mut rx_buf, &mut tx_buf, fan, UartName::B);

    uart_b_adapter.start().await;
}
