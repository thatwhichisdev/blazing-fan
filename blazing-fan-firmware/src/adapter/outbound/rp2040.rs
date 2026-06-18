use crate::core::port::outbound::mcu::{Mcu, McuError};

use ariel_os::gpio::Output;
use embassy_rp::adc::{Adc, Blocking, Channel};

pub struct Rp2040<'a> {
    adc: Adc<'a, Blocking>,
    temp_ch: Channel<'a>,
    vsys_ch: Channel<'a>,
    led_output: Output<'a>,
}

impl<'a> Rp2040<'a> {
    pub fn new(
        adc: Adc<'a, Blocking>,
        temp_ch: Channel<'a>,
        vsys_ch: Channel<'a>,
        led_output: Output<'a>,
    ) -> Self {
        Self {
            adc,
            temp_ch,
            vsys_ch,
            led_output,
        }
    }
}

impl<'a> Mcu for Rp2040<'a> {
    fn get_internal_temp(&mut self) -> Result<i8, McuError> {
        let adc_raw = self.adc.blocking_read(&mut self.temp_ch)?;
        let adc_voltage = adc_raw as f32 * 3.3 / 4096.0;
        let temp_c = 27.0 - (adc_voltage - 0.706) / 0.001721;

        let temp_c_i8 = if temp_c < 0.0 {
            (temp_c - 0.5) as i8
        } else {
            (temp_c + 0.5) as i8
        };

        Ok(temp_c_i8)
    }

    fn get_system_voltage(&mut self) -> Result<u16, McuError> {
        let adc_raw = self.adc.blocking_read(&mut self.vsys_ch)?;
        let adc_voltage = (adc_raw as f32) * 3.3 * 3.0 / 4096.0;
        let vol_mv = ((adc_voltage * 1000.0) + 0.5) as u16;

        Ok(vol_mv)
    }

    fn led_on(&mut self) {
        self.led_output.set_high();
    }

    fn led_off(&mut self) {
        self.led_output.set_low();
    }
}
