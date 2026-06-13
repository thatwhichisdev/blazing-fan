#[derive(Debug, defmt::Format)]
pub enum Emc2101Error {
    Empty,
}

pub trait Emc2101Port {
    async fn fan_rpm(&mut self) -> Result<u16, Emc2101Error>;
}
