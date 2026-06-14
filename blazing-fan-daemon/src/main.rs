use std::time::Duration;

use blazing_fan_proto::{UartQuery, UartRequest};
use serial2_tokio::SerialPort;
use tokio::time::interval;
use zbus::conn::Builder;

use crate::{
    adapter::{inbound::dbus_adapter::DbusAdapter, outbound::uart_adapter::UartAdapter},
    core::port::outbound::uart_port::UartPort,
};

mod adapter;
mod config;
mod core;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let config = config::load_config()?;

    // Bootstrap uart adapter
    let port = SerialPort::open(config.uart.path, config.uart.baud_rate).unwrap();
    let mut uart_adapter = UartAdapter::new(port);

    // Boostrap dbus adapter
    let _connection = Builder::session()?
        .name("dev.thatwhichis.daemon")?
        .serve_at("/dev/thatwhichis/daemon", DbusAdapter)?
        .build()
        .await?;

    let mut ticker = interval(Duration::from_millis(config.polling.interval_ms));

    loop {
        ticker.tick().await;

        match uart_adapter
            .request(UartRequest::Query(UartQuery::FanGetStatus))
            .await
        {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}
