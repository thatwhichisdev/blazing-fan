#[cfg(context = "rp2040")]
use ariel_os::hal::{define_peripherals, group_peripherals, peripherals};

#[cfg(context = "rp2040")]
define_peripherals!(ButtonPin { button: PIN_12 });

#[cfg(context = "rp2040")]
define_peripherals!(UartAPins {
    uart0_tx: PIN_0,
    uart0_rx: PIN_1,
});

#[cfg(context = "rp2040")]
define_peripherals!(UartBPins {
    uart1_tx: PIN_8,
    uart1_rx: PIN_9
});

#[cfg(context = "rp2040")]
define_peripherals!(EmcPins {
    sda: PIN_4,
    scl: PIN_5,
});

#[cfg(context = "rp2040")]
define_peripherals!(PixelPins {
    pio: PIO0,
    led: PIN_15,
    dma: DMA_CH0,
});

#[cfg(context = "rp2040")]
define_peripherals!(FanPowerPin { pwr: PIN_16 });

#[cfg(context = "rp2040")]
define_peripherals!(PicoPins {
    adc: ADC,
    adc_tmp: ADC_TEMP_SENSOR,
    usb: PIN_24,
    vsys: PIN_29,
    led: PIN_25,
});

#[cfg(context = "rp2040")]
group_peripherals!(Peripherals {
    btn_pin: ButtonPin,
    emc_pins: EmcPins,
    fan_pwr_pin: FanPowerPin,
    pico_pins: PicoPins,
    pixel_pins: PixelPins,
    uart_a_pins: UartAPins,
    uart_b_pins: UartBPins,
});
