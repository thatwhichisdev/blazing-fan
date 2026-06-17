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
        sysinfo::{SysInfo, SystemFetcher},
    },
};

use blazing_fan_proto::{UartQuery, UartRequest};
use std::time::Duration;
use tokio::{
    sync::watch::{self, Receiver, Sender},
    time::interval,
};
use tokio_util::sync::CancellationToken;

mod adapter;
mod core;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let config = core::config::load_config()?;

    let _con = DbusAdapter::build_connection().await?;
    let syst_fetcher = SystemFetcher::new();
    let uart_adapter = UartAdapter::new(&config.uart)?;
    let otel_adapter = OtelAdapter::new(&config.otel)?;

    let (tx, _) = watch::channel(SysInfo::default());
    let uart_rx = tx.subscribe();
    let otel_rx = tx.subscribe();

    let cancellation = CancellationToken::new();
    let uart_cancellation = cancellation.child_token();
    let otel_cancellation = cancellation.child_token();

    let syst_task = tokio::spawn(syst_task(syst_fetcher, tx));
    let uart_task = tokio::spawn(uart_task(uart_adapter, uart_rx, uart_cancellation));
    let otel_task = tokio::spawn(otel_task(otel_adapter, otel_rx, otel_cancellation));

    tokio::select!(
        result = syst_task => {
            tracing::error!("system_info_task exited {:?}", result);
        }
        result = uart_task => {
            tracing::error!("uart_task exited {:?}", result);
        }
        result = otel_task => {
            tracing::error!("otel_task exited {:?}", result);
        }
        result = tokio::signal::ctrl_c() => {
            tracing::info!("shutdown requested: {:?}", result);
            cancellation.cancel();
        }
    );

    Ok(())
}

async fn syst_task(mut fetcher: SystemFetcher, tx: Sender<SysInfo>) {
    let mut ticker = interval(Duration::from_secs(9));

    loop {
        ticker.tick().await;
        let sys_info = fetcher.fetch();

        // todo: handle possible error
        tx.send(sys_info).unwrap();
    }
}

async fn uart_task(
    mut adapter: UartAdapter,
    rx: Receiver<SysInfo>,
    cancellation: CancellationToken,
) {
    let mut ticker = interval(Duration::from_secs(9));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let _sys_info = rx.borrow().to_owned();

                match adapter
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
            _ = cancellation.cancelled() => {
                if let Err(e) = adapter.shutdown().await {
                    tracing::error!("error during uart shutdown {:?}", e);
                };
                return;
            }
        }
    }
}

async fn otel_task(
    mut adapter: OtelAdapter,
    rx: Receiver<SysInfo>,
    cancellation: CancellationToken,
) {
    let mut ticker = interval(Duration::from_secs(9));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let sys_info = rx.borrow().to_owned();
                adapter.record_sys_info(&sys_info);
            }
            _ = cancellation.cancelled() => {
                if let Err(e) = adapter.shutdown() {
                    tracing::error!("error during otel shutdown {:?}", e);
                };
                return;
            }
        }
    }
}
