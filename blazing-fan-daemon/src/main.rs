use crate::{
    adapter::{
        inbound::dbus_adapter::DbusAdapter,
        outbound::{otel_adapter::OtelAdapter, uart_adapter::UartAdapter},
    },
    core::{
        port::outbound::otel_port::OtelPort, port::outbound::uart_port::UartPort,
        sysinfo::SysInfoFetcher,
    },
};

use blazing_fan_proto::{UartQuery, UartRequest};
use std::time::Duration;
use tokio::time::interval;

mod adapter;
mod core;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let config = core::config::load_config()?;

    // Bootstrap outbound otel adapter
    let otel_adapter = OtelAdapter::new(&config.otel)?;

    // Bootstrap outbound uart adapter
    let _uart_adapter = UartAdapter::new(&config.uart);

    // Bootstrap inbound dbus adapter
    let _con = DbusAdapter::build_connection().await?;

    // Bootstrap sys info fetcher
    let mut sys_info_fetcher = SysInfoFetcher::new();

    let mut ticker = interval(Duration::from_millis(config.polling.interval_ms));

    loop {
        ticker.tick().await;

        let sys_info = sys_info_fetcher.fetch();

        otel_adapter.record_sys_info(&sys_info);

        // match uart_adapter
        //     .request(UartRequest::Query(UartQuery::FanGetStatus))
        //     .await
        // {
        //     Ok(_) => todo!(),
        //     Err(_) => todo!(),
        // };
    }
}
