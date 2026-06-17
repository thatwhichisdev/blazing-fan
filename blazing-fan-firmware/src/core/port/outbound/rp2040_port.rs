use thiserror::Error;

#[derive(Error, Debug, defmt::Format)]
pub enum RP2040Error {
    #[error("adc conversion failed")]
    AdcError(embassy_rp::adc::Error),
}

impl From<embassy_rp::adc::Error> for RP2040Error {
    fn from(value: embassy_rp::adc::Error) -> Self {
        Self::AdcError(value)
    }
}

pub trait RP2040Port {
    fn board_tmp(&mut self) -> Result<i8, RP2040Error>;

    fn board_sys_voltage(&mut self) -> Result<f32, RP2040Error>;

    fn led_on(&mut self);

    fn led_off(&mut self);
}
