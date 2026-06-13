use crate::ports::emc2101_port::{Emc2101Error, Emc2101Port};
use ariel_os::hal::i2c::controller::I2c;
use emc2101::AsyncEMC2101;
use fugit::Rate;

pub struct Emc2101Adapter {
    emc: AsyncEMC2101<I2c>,
}

impl Emc2101Adapter {
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

impl Emc2101Port for Emc2101Adapter {
    async fn fan_rpm(&mut self) -> Result<u16, crate::ports::emc2101_port::Emc2101Error> {
        match self.emc.fan_rpm().await {
            Ok(rpm) => Ok(rpm),
            Err(_) => Err(Emc2101Error::Empty),
        }
    }
}
