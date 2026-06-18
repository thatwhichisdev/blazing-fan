use thiserror::Error;

#[derive(Error, Debug, defmt::Format)]
pub enum McuError {
    #[error("adc conversion failed")]
    AdcError(embassy_rp::adc::Error),
}

impl From<embassy_rp::adc::Error> for McuError {
    fn from(value: embassy_rp::adc::Error) -> Self {
        Self::AdcError(value)
    }
}

pub trait Mcu {
    fn get_internal_temp(&mut self) -> Result<i8, McuError>;

    fn get_system_voltage(&mut self) -> Result<u16, McuError>;

    fn led_on(&mut self);

    fn led_off(&mut self);
}
