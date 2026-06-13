use smart_leds::RGB8;

pub trait WS2812Port {
    async fn set_rgb8(&mut self, colors: [RGB8; 2]);
}
