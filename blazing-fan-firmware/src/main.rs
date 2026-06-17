#![no_main]
#![no_std]

mod adapter;
mod core;
mod pins;

use crate::{
    adapter::{
        inbound::{
            button_adapter::ButtonAdapter,
            uart_adapter::{UartAdapter, UartName},
        },
        outbound::{
            emc2101_adapter::Emc2101Adapter, fan_power_adapter::FanPowerAdapter,
            rp2040_adapter::RP2040Adapter, ws2812_adapter::WS2812Adapter,
        },
    },
    core::Fan,
    pins::{
        ButtonPin, EmcPins, FanPowerPin, Peripherals, PicoPins, UartAPins, UartBPins, WS2812Pins,
    },
};
use ariel_os::{
    asynch::Spawner,
    gpio::{Input, Level, Output},
    hal::{i2c, uart},
    reexports::embassy_time::Ticker,
    time::Duration,
};
use blazing_fan_proto::{UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE};
use embassy_rp::{
    adc, bind_interrupts,
    peripherals::PIO0,
    pio::Pio,
    pio_programs::ws2812::{PioWs2812, PioWs2812Program},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, watch::Watch};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
});

/// Fan should be wrapped in Mutex since we're going to share it across multiple tasks
static FAN: StaticCell<
    Mutex<
        CriticalSectionRawMutex,
        Fan<RP2040Adapter, Emc2101Adapter, FanPowerAdapter, WS2812Adapter>,
    >,
> = StaticCell::new();

#[allow(clippy::type_complexity)]
static FAN_READY: Watch<
    CriticalSectionRawMutex,
    &'static Mutex<
        CriticalSectionRawMutex,
        Fan<RP2040Adapter, Emc2101Adapter, FanPowerAdapter, WS2812Adapter>,
    >,
    4,
> = Watch::new();

#[ariel_os::spawner(autostart, peripherals)]
fn boot(spawner: Spawner, peripherals: Peripherals) {
    spawner
        .spawn(core_task(
            peripherals.pico_pins,
            peripherals.emc_pins,
            peripherals.fan_pwr_pin,
            peripherals.ws_pins,
        ))
        .expect("single instance of this task is running");

    spawner
        .spawn(button_adapter_task(peripherals.btn_pin))
        .expect("single instance of this task is running");

    spawner
        .spawn(uart_a_adapter_task(peripherals.uart_a_pins))
        .expect("single instance of this task is running");

    spawner
        .spawn(uart_b_adapter_task(peripherals.uart_b_pins))
        .expect("single instance of this task is running");

    spawner
        .spawn(ticker())
        .expect("single instance of this task is running");
}

#[ariel_os::task]
async fn core_task(
    pico_pins: PicoPins,
    emc_pins: EmcPins,
    fan_pwr_pin: FanPowerPin,
    ws_pins: WS2812Pins,
) {
    defmt::info!("Booting firmware");

    // Bootstraping RP2040 adapter
    let adc_config = adc::Config::default();
    let adc = adc::Adc::new_blocking(pico_pins.adc, adc_config);
    let tmp_ch = adc::Channel::new_temp_sensor(pico_pins.adc_tmp);
    let vsys_ch = adc::Channel::new_pin(pico_pins.vsys, embassy_rp::gpio::Pull::None);
    let led = Output::new(pico_pins.led, Level::Low);
    let rp2040_adapter = RP2040Adapter::new(adc, tmp_ch, vsys_ch, led);

    // Bootstraping fan power adapter
    let fan_power_output = Output::new(fan_pwr_pin.pwr, Level::High);
    let fan_power_adapter = FanPowerAdapter::new(fan_power_output);

    // Bootstraping EMC2101 adapter
    let i2c_config = i2c::controller::Config::default();
    let i2c0 = i2c::controller::I2C0::new(emc_pins.sda, emc_pins.scl, i2c_config);
    let emc2101_adapter = Emc2101Adapter::new(i2c0).await;

    // Bootstraping WS2812 adapter
    let mut pio = Pio::new(ws_pins.pio, Irqs);
    let pio_program = PioWs2812Program::new(&mut pio.common);
    let ws2812 = PioWs2812::<PIO0, 0, 2>::new(
        &mut pio.common,
        pio.sm0,
        ws_pins.dma,
        ws_pins.led,
        &pio_program,
    );
    let ws2812_adapter = WS2812Adapter::new(ws2812);

    // Bootstraping Fan
    let fan: &'static _ = FAN.init(Mutex::new(Fan::new(
        rp2040_adapter,
        emc2101_adapter,
        fan_power_adapter,
        ws2812_adapter,
    )));

    let fan_signal_sender = FAN_READY.dyn_sender();
    fan_signal_sender.send(fan);
}

#[ariel_os::task]
async fn ticker() {
    defmt::info!("Booting ticker");

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().expect("receivers are defined");
    let fan = fan_signal_rcv.get().await;

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let mut guard = fan.lock().await;
        if let Err(e) = guard.tick().await {
            defmt::error!("error happened in fan main duty cycle {:?}", e);
        };
    }
}

#[ariel_os::task]
async fn button_adapter_task(pins: ButtonPin) {
    defmt::info!("Booting button listener");

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().expect("receivers are defined");
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

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().expect("receivers are defined");
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

    let mut fan_signal_rcv = FAN_READY.dyn_receiver().expect("receivers are defined");
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
