pub trait ButtonPort {
    async fn btn_pressed(&mut self);
}
