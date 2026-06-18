use crate::core::port::outbound::fan_controller::{FanController, FanControllerError};
use ariel_os::hal::i2c::controller::I2c;
use bounded_integer::BoundedU8;
use emc2101::AsyncEMC2101;
use fugit::Rate;

pub struct Emc2101 {
    emc: AsyncEMC2101<I2c>,
}

impl Emc2101 {
    pub async fn new(i2c: I2c) -> Self {
        defmt::info!("EMC2101: Booting");
        let mut emc = AsyncEMC2101::new(i2c).await.unwrap();

        defmt::info!("EMC2101: Tach input enabled");
        emc.enable_tach_input().await.unwrap();

        defmt::info!("EMC2101: Set pulse width modulation to 25kHz");
        emc.set_fan_pwm(Rate::<u32, _, _>::kHz(25), false)
            .await
            .unwrap();

        Self { emc }
    }
}

impl FanController for Emc2101 {
    async fn get_fan_rpm(&mut self) -> Result<u16, FanControllerError> {
        match self.emc.fan_rpm().await {
            Ok(rpm) => Ok(rpm),
            Err(_) => Err(FanControllerError::Empty),
        }
    }

    async fn get_fan_tmp_external(&mut self) -> Result<i8, FanControllerError> {
        match self.emc.temp_external().await {
            Ok(tmp) => Ok(tmp),
            Err(_) => Err(FanControllerError::Empty),
        }
    }

    async fn get_fan_tmp_internal(&mut self) -> Result<i8, FanControllerError> {
        match self.emc.temp_internal().await {
            Ok(tmp) => Ok(tmp),
            Err(_) => Err(FanControllerError::Empty),
        }
    }

    async fn set_fan_power(&mut self, val: BoundedU8<0, 63>) -> Result<(), FanControllerError> {
        match self.emc.set_fan_power(val).await {
            Ok(_) => Ok(()),
            Err(_) => Err(FanControllerError::Empty),
        }
    }

    async fn set_fan_power_max(&mut self) -> Result<(), FanControllerError> {
        match self.emc.set_fan_power(BoundedU8::MAX).await {
            Ok(_) => Ok(()),
            Err(_) => Err(FanControllerError::Empty),
        }
    }

    async fn set_fan_power_min(&mut self) -> Result<(), FanControllerError> {
        match self.emc.set_fan_power(BoundedU8::MIN).await {
            Ok(_) => Ok(()),
            Err(_) => Err(FanControllerError::Empty),
        }
    }
}
