use bounded_integer::BoundedU8;
use thiserror::Error;

#[derive(Error, Debug, defmt::Format)]
pub enum FanControllerError {
    #[error("i2c bus error")]
    I2c,
    #[error("emc2101 product id is not supported")]
    InvalidDevice,
    #[error("emc2101 value is invalid")]
    InvalidValue,
    #[error("emc2101 data size is invalid")]
    InvalidSize,
    #[error("emc2101 lookup table is not sorted")]
    InvalidSorting,
    #[error("emc2101 internal driver error")]
    Internal,
}

impl From<emc2101::Error<ariel_os::i2c::controller::Error>> for FanControllerError {
    fn from(error: emc2101::Error<ariel_os::i2c::controller::Error>) -> Self {
        match error {
            emc2101::Error::I2c(_) => Self::I2c,
            emc2101::Error::InvalidID => Self::InvalidDevice,
            emc2101::Error::InvalidValue => Self::InvalidValue,
            emc2101::Error::InvalidSize => Self::InvalidSize,
            emc2101::Error::InvalidSorting => Self::InvalidSorting,
            emc2101::Error::Internal => Self::Internal,
        }
    }
}

pub trait FanController {
    async fn get_fan_rpm(&mut self) -> Result<u16, FanControllerError>;

    async fn get_fan_tmp_external(&mut self) -> Result<i8, FanControllerError>;

    async fn get_fan_tmp_internal(&mut self) -> Result<i8, FanControllerError>;

    async fn set_fan_power(&mut self, val: BoundedU8<0, 63>) -> Result<(), FanControllerError>;

    async fn set_fan_power_max(&mut self) -> Result<(), FanControllerError>;

    async fn set_fan_power_min(&mut self) -> Result<(), FanControllerError>;

    async fn set_fan_auto(&mut self) -> Result<(), FanControllerError>;
}
