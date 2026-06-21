use crate::core::port::outbound::fan_controller::{FanController, FanControllerError};
use ariel_os::hal::i2c::controller::I2c;
use bounded_integer::BoundedU8;
use emc2101::{AsyncEMC2101, Level};
use fugit::Rate;
use heapless::Vec;
use static_cell::StaticCell;

static FAN_LUT: StaticCell<Vec<Level, 8>> = StaticCell::new();

pub struct Emc2101<'a> {
    emc: AsyncEMC2101<I2c>,
    lut: &'a mut Vec<Level, 8>,
}

impl<'a> Emc2101<'a> {
    pub async fn new(i2c: I2c) -> Result<Self, FanControllerError> {
        let mut emc = AsyncEMC2101::new(i2c).await?;
        let lut = FAN_LUT.init(Self::lut());

        emc.enable_tach_input().await?;
        emc.set_fan_pwm(
            Rate::<u32, _, _>::Hz(22_500),
            false,
            Some(BoundedU8::<1, 31>::new(8).unwrap()),
        )
        .await?;

        Ok(Self { emc, lut })
    }

    fn lut_level(temp: u8, step: u8) -> Level {
        Level {
            temp: BoundedU8::new(temp).expect("LUT temp must be within 0..=127"),
            step: BoundedU8::new(step).expect("LUT fan step must be within 0..=63"),
        }
    }

    fn lut() -> Vec<Level, 8> {
        let mut lut = Vec::<Level, 8>::new();

        lut.push(Self::lut_level(1, 3))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(20, 4))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(25, 6))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(30, 7))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(35, 9))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(40, 11))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(45, 14))
            .expect("LUT must fit into 8 entries");
        lut.push(Self::lut_level(50, 16))
            .expect("LUT must fit into 8 entries");

        lut
    }
}

impl<'a> FanController for Emc2101<'a> {
    async fn get_fan_rpm(&mut self) -> Result<u16, FanControllerError> {
        self.emc.fan_rpm().await.map_err(|e| e.into())
    }

    async fn get_fan_tmp_external(&mut self) -> Result<i8, FanControllerError> {
        self.emc.temp_external().await.map_err(|e| e.into())
    }

    async fn get_fan_tmp_internal(&mut self) -> Result<i8, FanControllerError> {
        self.emc.temp_internal().await.map_err(|e| e.into())
    }

    async fn set_fan_power(&mut self, val: BoundedU8<0, 63>) -> Result<(), FanControllerError> {
        self.emc
            .set_fan_power(val)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }

    async fn set_fan_power_max(&mut self) -> Result<(), FanControllerError> {
        self.emc
            .set_fan_power(BoundedU8::MAX)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }

    async fn set_fan_power_min(&mut self) -> Result<(), FanControllerError> {
        self.emc
            .set_fan_power(BoundedU8::MIN)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }

    async fn set_fan_auto(&mut self) -> Result<(), FanControllerError> {
        let lut = (*self.lut).clone();
        let hysteresis = BoundedU8::new(4).expect("lut hysteresis must be within 0..=31");

        self.emc
            .set_fan_lut(lut, hysteresis)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
