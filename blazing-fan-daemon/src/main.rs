use std::time::Duration;

use serial2_tokio::SerialPort;
use sysinfo::{Components, System};
use tokio::time::interval;
use zbus::conn::Builder;

use crate::{
    adapter::{
        inbound::dbus_adapter::DbusAdapter,
        outbound::{otel_adapter::OtelAdapter, uart_adapter::UartAdapter},
    },
    core::port::outbound::otel_port::OtelPort,
};

mod adapter;
mod config;
mod core;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let config = config::load_config()?;

    // Bootstrap outbound otel adapter
    let otel_adapter = OtelAdapter::new(config.otel)?;

    // Bootstrap outbound uart adapter
    let port = SerialPort::open(config.uart.path, config.uart.baud_rate).unwrap();
    let _uart_adapter = UartAdapter::new(port);

    // Boostrap inbound dbus adapter
    let _connection = Builder::session()?
        .name("dev.thatwhichis.daemon")?
        .serve_at("/dev/thatwhichis/daemon", DbusAdapter)?
        .build()
        .await?;

    let mut ticker = interval(Duration::from_millis(config.polling.interval_ms));
    let mut sys = System::new_all();
    let mut components = Components::new_with_refreshed_list();
    let cpu_tmp = components
        .iter_mut()
        .find(|c| c.id() == Some("hwmon0_1"))
        .unwrap();

    loop {
        sys.refresh_all();
        cpu_tmp.refresh();

        otel_adapter.record_cpu_load(sys.global_cpu_usage() as f64);
        otel_adapter.record_mem_usg(sys.used_memory());

        cpu_tmp.temperature().map(|tmp| {
            otel_adapter.record_cpu_tmp(tmp as f64);
            tracing::info!("cpu temp: {}°C", tmp);
        });

        // match uart_adapter
        //     .request(UartRequest::Query(UartQuery::FanGetStatus))
        //     .await
        // {
        //     Ok(_) => todo!(),
        //     Err(_) => todo!(),
        // }

        ticker.tick().await;
    }
}
