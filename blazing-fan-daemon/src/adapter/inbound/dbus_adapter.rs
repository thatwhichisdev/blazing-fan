use zbus::interface;

use crate::core::port::inbound::dbus_port::DbusPort;

pub struct DbusAdapter;

#[interface(name = "dev.thatwhichis.daemon")]
impl DbusPort for DbusAdapter {
    async fn greet(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }
}
