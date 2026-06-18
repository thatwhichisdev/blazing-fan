use ariel_os::gpio::Output;

use crate::core::port::outbound::fan_supply::FanSupply;

pub struct GpioFanSupply<'a> {
    output: Output<'a>,
}

impl<'a> GpioFanSupply<'a> {
    pub fn new(output: Output<'a>) -> Self {
        Self { output }
    }
}

impl<'a> FanSupply for GpioFanSupply<'a> {
    fn enable(&mut self) {
        self.output.set_high();
    }

    fn disable(&mut self) {
        self.output.set_low();
    }
}
