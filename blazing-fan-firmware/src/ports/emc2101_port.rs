use bounded_integer::BoundedU8;

#[derive(Debug, defmt::Format)]
pub enum Emc2101Error {
    Empty,
}

pub trait Emc2101Port {
    async fn fan_rpm(&mut self) -> Result<u16, Emc2101Error>;

    async fn fan_tmp_external(&mut self) -> Result<i8, Emc2101Error>;

    async fn fan_tmp_internal(&mut self) -> Result<i8, Emc2101Error>;

    async fn set_fan_power(&mut self, val: BoundedU8<0, 63>) -> Result<(), Emc2101Error>;
}
