use embassy_rp::{peripherals::PIO0, pio_programs::ws2812::PioWs2812};
use smart_leds::RGB8;

use crate::core::port::outbound::ws2812_port::WS2812Port;

pub struct WS2812Adapter<'a> {
    pxl: PioWs2812<'a, PIO0, 0, 2>,
}

impl<'a> WS2812Adapter<'a> {
    pub fn new(pxl: PioWs2812<'a, PIO0, 0, 2>) -> Self {
        Self { pxl }
    }
}

impl<'a> WS2812Port for WS2812Adapter<'a> {
    async fn set_rgb8(&mut self, colors: [RGB8; 2]) {
        self.pxl.write(&colors).await;
    }
}
