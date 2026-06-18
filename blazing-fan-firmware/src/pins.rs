use ariel_os::hal::{define_peripherals, group_peripherals, peripherals};

#[cfg(context = "rp2040")]
define_peripherals!(UserButtonPin { input: PIN_12 });

#[cfg(context = "rp2040")]
define_peripherals!(Uart0Pins {
    uart0_tx: PIN_0,
    uart0_rx: PIN_1,
});

#[cfg(context = "rp2040")]
define_peripherals!(Uart1Pins {
    uart1_tx: PIN_8,
    uart1_rx: PIN_9
});

#[cfg(context = "rp2040")]
define_peripherals!(FanControllerPins {
    sda: PIN_4,
    scl: PIN_5,
});

#[cfg(context = "rp2040")]
define_peripherals!(StatusIndicatorPins {
    pio: PIO0,
    led: PIN_15,
    dma: DMA_CH0,
});

#[cfg(context = "rp2040")]
define_peripherals!(FanSupplyPin { pwr: PIN_16 });

#[cfg(context = "rp2040")]
define_peripherals!(McuPins {
    adc: ADC,
    temp_ch: ADC_TEMP_SENSOR,
    vsys_ch: PIN_29,
    led_output: PIN_25,
});

#[cfg(context = "rp2040")]
group_peripherals!(Peripherals {
    user_btn_pin: UserButtonPin,
    fan_ctrl_pins: FanControllerPins,
    fan_supply_pin: FanSupplyPin,
    mcu_pins: McuPins,
    status_indicator_pins: StatusIndicatorPins,
    uart_0_pins: Uart0Pins,
    uart_1_pins: Uart1Pins,
});
