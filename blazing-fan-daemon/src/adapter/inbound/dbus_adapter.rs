use zbus::{Connection, conn::Builder, interface};

use crate::core::port::inbound::dbus_port::DbusPort;

pub struct DbusAdapter {}

impl DbusAdapter {
    pub async fn build_connection() -> color_eyre::Result<Connection> {
        let dbus_adapter = DbusAdapter {};
        let con = Builder::session()?
            .name("dev.thatwhichis.daemon")?
            .serve_at("/dev/thatwhichis/daemon", dbus_adapter)?
            .build()
            .await?;

        Ok(con)
    }
}

#[interface(name = "dev.thatwhichis.daemon")]
impl DbusPort for DbusAdapter {
    async fn greet(&self, name: &str) -> String {
        format!("Hello {name}!")
    }
}
