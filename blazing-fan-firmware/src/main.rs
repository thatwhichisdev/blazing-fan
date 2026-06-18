#![no_main]
#![no_std]

mod adapter;
mod core;
mod pins;

use crate::{
    adapter::{
        inbound::{gpio_button::GpioButton, uart::UartAdapter},
        outbound::{
            emc2101::Emc2101, gpio_fan_supply::GpioFanSupply, rp2040::Rp2040, ws2812::Ws2812,
        },
    },
    core::{System, port::inbound::uart_port::UartName},
    pins::{
        FanControllerPins, FanSupplyPin, McuPins, Peripherals, StatusIndicatorPins, Uart0Pins,
        Uart1Pins, UserButtonPin,
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

static SYSTEM: StaticCell<
    Mutex<CriticalSectionRawMutex, System<Rp2040, Emc2101, GpioFanSupply, Ws2812>>,
> = StaticCell::new();

#[allow(clippy::type_complexity)]
static SYSTEM_READY_SIGNAL: Watch<
    CriticalSectionRawMutex,
    &'static Mutex<CriticalSectionRawMutex, System<Rp2040, Emc2101, GpioFanSupply, Ws2812>>,
    4,
> = Watch::new();

#[ariel_os::spawner(autostart, peripherals)]
fn boot(spawner: Spawner, peripherals: Peripherals) {
    spawner
        .spawn(core_task(
            peripherals.mcu_pins,
            peripherals.fan_ctrl_pins,
            peripherals.fan_supply_pin,
            peripherals.status_indicator_pins,
        ))
        .expect("failed to spawn core_task: task instance already running");

    spawner
        .spawn(button_adapter_task(peripherals.user_btn_pin))
        .expect("failed to spawn button_adapter_task: task instance already running");

    spawner
        .spawn(uart_a_adapter_task(peripherals.uart_0_pins))
        .expect("failed to spawn uart_a_adapter_task: task instance already running");

    spawner
        .spawn(uart_b_adapter_task(peripherals.uart_1_pins))
        .expect("failed to spawn uart_b_adapter_task: task instance already running");

    spawner
        .spawn(ticker())
        .expect("failed to spawn ticker task: task instance already running");
}

#[ariel_os::task]
async fn core_task(
    pico_pins: McuPins,
    emc_pins: FanControllerPins,
    fan_pwr_pin: FanSupplyPin,
    ws_pins: StatusIndicatorPins,
) {
    defmt::info!("Booting firmware");

    let adc_config = adc::Config::default();
    let adc = adc::Adc::new_blocking(pico_pins.adc, adc_config);
    let temp_ch = adc::Channel::new_temp_sensor(pico_pins.temp_ch);
    let vsys_ch = adc::Channel::new_pin(pico_pins.vsys_ch, embassy_rp::gpio::Pull::None);
    let led_output = Output::new(pico_pins.led_output, Level::Low);
    let rp2040 = Rp2040::new(adc, temp_ch, vsys_ch, led_output);

    let fan_supply_output = Output::new(fan_pwr_pin.pwr, Level::High);
    let fan_supply = GpioFanSupply::new(fan_supply_output);

    let i2c_config = i2c::controller::Config::default();
    let i2c0 = i2c::controller::I2C0::new(emc_pins.sda, emc_pins.scl, i2c_config);
    let emc2101 = Emc2101::new(i2c0).await;

    let mut pio = Pio::new(ws_pins.pio, Irqs);
    let pio_program = PioWs2812Program::new(&mut pio.common);
    let pio_driver = PioWs2812::<PIO0, 0, 2>::new(
        &mut pio.common,
        pio.sm0,
        ws_pins.dma,
        ws_pins.led,
        &pio_program,
    );
    let ws2812 = Ws2812::new(pio_driver);

    let system: &'static _ =
        SYSTEM.init(Mutex::new(System::new(rp2040, emc2101, fan_supply, ws2812)));

    SYSTEM_READY_SIGNAL.dyn_sender().send(system);
}

#[ariel_os::task]
async fn ticker() {
    let system = SYSTEM_READY_SIGNAL
        .dyn_receiver()
        .expect("receiver capacity exceeded, expected at most 4 system consumers")
        .get()
        .await;

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let mut guard = system.lock().await;
        if let Err(e) = guard.tick().await {
            defmt::error!("error happened in fan main duty cycle {:?}", e);
        };
    }
}

#[ariel_os::task]
async fn button_adapter_task(pins: UserButtonPin) {
    let system = SYSTEM_READY_SIGNAL
        .dyn_receiver()
        .expect("receiver capacity exceeded, expected at most 4 system consumers")
        .get()
        .await;

    let button_input = Input::builder(pins.input, ariel_os::gpio::Pull::Up)
        .build_with_interrupt()
        .unwrap();

    let mut button_adapter = GpioButton::new(button_input, system);
    button_adapter.start().await;
}

#[ariel_os::task]
async fn uart_a_adapter_task(pins: Uart0Pins) {
    let system = SYSTEM_READY_SIGNAL
        .dyn_receiver()
        .expect("receiver capacity exceeded, expected at most 4 system consumers")
        .get()
        .await;

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
    .expect("UART0::new returned an error despite being documented as infallible");

    let mut uart_a_adapter = UartAdapter::new(uart, &mut rx_buf, &mut tx_buf, system, UartName::A);

    uart_a_adapter.start().await;
}

#[ariel_os::task]
async fn uart_b_adapter_task(pins: Uart1Pins) {
    let system = SYSTEM_READY_SIGNAL
        .dyn_receiver()
        .expect("receiver capacity exceeded, expected at most 4 system consumers")
        .get()
        .await;

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
    .expect("UART1::new returned an error despite being documented as infallible");

    let mut uart_b_adapter = UartAdapter::new(uart, &mut rx_buf, &mut tx_buf, system, UartName::B);

    uart_b_adapter.start().await;
}
