pub trait StatusIndicator {
    async fn set_green(&mut self);

    async fn set_orange(&mut self);

    async fn set_none(&mut self);
}
