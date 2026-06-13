pub trait FanPowerPort {
    fn pwr_on(&mut self);

    fn pwr_off(&mut self);
}
