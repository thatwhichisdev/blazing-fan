use smart_leds::RGB8;

pub trait StatusIndicator {
    async fn set_custom(&mut self, colors: [RGB8; 2]);

    async fn set_green(&mut self);

    async fn set_orange(&mut self);

    async fn set_red(&mut self);

    async fn set_none(&mut self);
}
