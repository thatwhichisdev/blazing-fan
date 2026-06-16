use crate::{
    adapter::{
        inbound::dbus_adapter::DbusAdapter,
        outbound::{otel_adapter::OtelAdapter, uart_adapter::UartAdapter},
    },
    core::{
        port::outbound::{
            otel_port::OtelPort,
            uart_port::{UartError, UartPort},
        },
        sysinfo::SystemFetcher,
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

    let mut sys_fetcher = SystemFetcher::new();
    let mut uart_adapter = UartAdapter::new(&config.uart)?;
    let _con = DbusAdapter::build_connection().await?;
    let otel_adapter = OtelAdapter::new(&config.otel)?;

    let mut ticker = interval(Duration::from_millis(config.polling.interval_ms));

    loop {
        ticker.tick().await;

        let sys_info = sys_fetcher.fetch();

        otel_adapter.record_sys_info(&sys_info);

        // todo: figure out how to implement a circuitbreaker or some other mechanism to determine if fan is available on the port
        //       it is possible to have a compute blade without a fan, in that case we don't want to spam uart bus with request and
        //       just perform ping/pong request once in a while to check if fan got connected
        match uart_adapter
            .request(UartRequest::Query(UartQuery::FanGetStatus))
            .await
        {
            Ok(res) => {
                tracing::info!("uart response received: {:?}", res);
            }
            Err(err) => match err {
                UartError::Timeout => tracing::warn!("uart request timed out"),
                UartError::IoError(e) => tracing::error!("uart request failed [io error: {:?}]", e),
                UartError::PostcardError(e) => {
                    tracing::error!("uart request failed [postcard error: {:?}]", e);
                }
            },
        }
    }
}
