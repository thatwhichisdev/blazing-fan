use bounded_integer::BoundedU8;
use thiserror::Error;

#[derive(Error, Debug, defmt::Format)]
pub enum FanControllerError {
    #[error("empty")]
    Empty,
}

pub trait FanController {
    async fn get_fan_rpm(&mut self) -> Result<u16, FanControllerError>;

    async fn get_fan_tmp_external(&mut self) -> Result<i8, FanControllerError>;

    async fn get_fan_tmp_internal(&mut self) -> Result<i8, FanControllerError>;

    async fn set_fan_power(&mut self, val: BoundedU8<0, 63>) -> Result<(), FanControllerError>;

    async fn set_fan_power_max(&mut self) -> Result<(), FanControllerError>;

    async fn set_fan_power_min(&mut self) -> Result<(), FanControllerError>;
}
