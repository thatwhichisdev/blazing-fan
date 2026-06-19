use embassy_rp::{peripherals::PIO0, pio_programs::ws2812::PioWs2812};
use smart_leds::{
    RGB8,
    colors::{self},
};

use crate::core::port::outbound::status_indicator::StatusIndicator;

static GREEN: [RGB8; 2] = [colors::GREEN; 2];
static ORANGE: [RGB8; 2] = [colors::ORANGE; 2];
static NONE: [RGB8; 2] = [colors::BLACK; 2];

pub struct Ws2812<'a> {
    driver: PioWs2812<'a, PIO0, 0, 2>,
}

impl<'a> Ws2812<'a> {
    pub fn new(driver: PioWs2812<'a, PIO0, 0, 2>) -> Self {
        Self { driver }
    }
}

impl<'a> StatusIndicator for Ws2812<'a> {
    async fn set_green(&mut self) {
        self.driver.write(&GREEN).await;
    }

    async fn set_orange(&mut self) {
        self.driver.write(&ORANGE).await;
    }

    async fn set_none(&mut self) {
        self.driver.write(&NONE).await;
    }
}
