use ariel_os::hal::{define_peripherals, group_peripherals, i2c, peripherals, uart};

#[cfg(context = "rp2040")]
define_peripherals!(ButtonPin { button: PIN_12 });

#[cfg(context = "rp2040")]
pub type UartA<'a> = uart::UART0<'a>;

#[cfg(context = "rp2040")]
define_peripherals!(UartAPins {
    uart0_tx: PIN_0,
    uart0_rx: PIN_1,
});

#[cfg(context = "rp2040")]
pub type UartB<'a> = uart::UART1<'a>;

#[cfg(context = "rp2040")]
define_peripherals!(UartBPins {
    uart1_tx: PIN_8,
    uart1_rx: PIN_9
});

#[cfg(context = "rp2040")]
pub type EmcI2C = i2c::controller::I2C0;

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
define_peripherals!(LedPin { led: PIN_25 });

#[cfg(context = "rp2040")]
define_peripherals!(PicoPins {
    adc: ADC,
    adc_tmp: ADC_TEMP_SENSOR,
    usb: PIN_24,
    vsys: PIN_29,
});

#[cfg(context = "rp2040")]
group_peripherals!(Peripherals {
    pico: PicoPins,
    emc: EmcPins,
    pixel: PixelPins,
    fan_pwr: FanPowerPin,
    led: LedPin
});
