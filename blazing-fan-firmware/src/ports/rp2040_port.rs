#[derive(Debug, defmt::Format)]
pub enum RP2040Error {}

pub trait RP2040Port {
    fn board_tmp(&mut self) -> Result<f32, RP2040Error>;

    fn board_sys_voltage(&mut self) -> Result<f32, RP2040Error>;

    fn led_on(&mut self);

    fn led_off(&mut self);
}
