use crate::core::port::outbound::rp2040_port::{RP2040Error, RP2040Port};

use ariel_os::gpio::Output;
use embassy_rp::adc::{Adc, Blocking, Channel};

pub struct RP2040Adapter<'a> {
    adc: Adc<'a, Blocking>,
    tmp_ch: Channel<'a>,
    vsys_ch: Channel<'a>,
    led: Output<'a>,
}

impl<'a> RP2040Adapter<'a> {
    pub fn new(
        adc: Adc<'a, Blocking>,
        tmp_ch: Channel<'a>,
        vsys_ch: Channel<'a>,
        led: Output<'a>,
    ) -> Self {
        Self {
            adc,
            tmp_ch,
            vsys_ch,
            led,
        }
    }
}

impl<'a> RP2040Port for RP2040Adapter<'a> {
    fn mcu_tmp(&mut self) -> Result<i8, RP2040Error> {
        let adc_raw = self.adc.blocking_read(&mut self.tmp_ch)?;
        let adc_voltage = adc_raw as f32 * 3.3 / 4096.0;
        let temp_c = 27.0 - (adc_voltage - 0.706) / 0.001721;

        let temp_c_i8 = if temp_c < 0.0 {
            (temp_c - 0.5) as i8
        } else {
            (temp_c + 0.5) as i8
        };

        Ok(temp_c_i8)
    }

    fn mcu_sys_vol_mv(&mut self) -> Result<u16, RP2040Error> {
        let adc_raw = self.adc.blocking_read(&mut self.vsys_ch)?;
        let adc_voltage = (adc_raw as f32) * 3.3 * 3.0 / 4096.0;
        let vol_mv = ((adc_voltage * 1000.0) + 0.5) as u16;

        Ok(vol_mv)
    }

    fn led_on(&mut self) {
        self.led.set_high();
    }

    fn led_off(&mut self) {
        self.led.set_low();
    }
}
