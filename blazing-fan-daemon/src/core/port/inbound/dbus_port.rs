pub trait DbusPort {
    async fn greet(&self, name: &str) -> String;
}
