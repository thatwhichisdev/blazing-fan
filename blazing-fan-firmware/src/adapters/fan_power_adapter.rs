use ariel_os::gpio::Output;

use crate::ports::fan_power_port::FanPowerPort;

pub struct FanPowerAdapter<'a> {
    pwr: Output<'a>,
}

impl<'a> FanPowerAdapter<'a> {
    pub fn new(pwr: Output<'a>) -> Self {
        Self { pwr }
    }
}

impl<'a> FanPowerPort for FanPowerAdapter<'a> {
    fn pwr_on(&mut self) {
        self.pwr.set_high();
    }

    fn pwr_off(&mut self) {
        self.pwr.set_low();
    }
}
