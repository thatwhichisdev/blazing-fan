use ariel_os::gpio::Output;
use embassy_rp::adc::{Adc, Blocking, Channel};

use crate::ports::rp2040_port::RP2040Port;

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
    fn board_tmp(&mut self) -> Result<f32, crate::ports::rp2040_port::RP2040Error> {
        let adc_raw = self.adc.blocking_read(&mut self.tmp_ch).unwrap();
        let adc_voltage = adc_raw as f32 * 3.3 / 4096.0;
        let temp = 27.0 - (adc_voltage - 0.706) / 0.001721;
        let sign = if temp < 0.0 { -1.0 } else { 1.0 };
        let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
        let temp = (rounded_temp_x10 as f32) / 10.0;

        defmt::debug!("temp adc_raw {=u16}", adc_raw);
        defmt::debug!("temp adc_voltage {=f32}", adc_voltage);
        defmt::info!("tmp {=f32}", temp);

        Ok(temp)
    }

    fn board_sys_voltage(&mut self) -> Result<f32, crate::ports::rp2040_port::RP2040Error> {
        let adc_raw = self.adc.blocking_read(&mut self.vsys_ch).unwrap();
        let adc_voltage = (adc_raw as f32) * 3.3 * 3.0 / 4096.0;

        defmt::debug!("vsys adc_raw {=u16}", adc_raw);
        defmt::info!("vsys adc_voltage {=f32}", adc_voltage);

        Ok(adc_voltage)
    }

    fn led_on(&mut self) {
        self.led.set_high();
    }

    fn led_off(&mut self) {
        self.led.set_low();
    }
}
