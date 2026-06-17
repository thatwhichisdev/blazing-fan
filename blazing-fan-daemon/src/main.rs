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
use sd_notify::NotifyState;
use std::time::Duration;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::watch::{self, Receiver, Sender},
    time::interval,
};
use tokio_util::sync::CancellationToken;

mod adapter;
mod core;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

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

    let mut sigintr = signal::unix::signal(SignalKind::interrupt())?;
    let mut sigquit = signal::unix::signal(SignalKind::quit())?;
    let mut sigterm = signal::unix::signal(SignalKind::terminate())?;

    let _ = sd_notify::notify(&[NotifyState::Ready])?;

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
        result = signal::ctrl_c() => {
            tracing::info!("shutdown requested manually: {:?}", result);
        }
        result = sigintr.recv() => {
            tracing::info!("SIGINT received, shutting down service gracefully {:?}", result);
        }
        result = sigquit.recv() => {
            tracing::info!("SIGQUIT received, shutting down service gracefully {:?}", result);
        }
        result = sigterm.recv() => {
            tracing::info!("SIGTERM received, shutting down service gracefully {:?}", result);
        }
    );

    let _ = sd_notify::notify(&[NotifyState::Stopping])?;
    cancellation.cancel();

    Ok(())
}

async fn syst_task(mut fetcher: SystemFetcher, tx: Sender<SysInfo>) {
    let mut ticker = interval(Duration::from_secs(9));

    loop {
        ticker.tick().await;
        let sys_info = fetcher.fetch();

        if let Err(e) = tx.send(sys_info) {
            tracing::error!("Failed to dispatch system info event, {:?}", e);
        };
    }
}

async fn uart_task(
    mut adapter: UartAdapter,
    mut rx: Receiver<SysInfo>,
    cancellation: CancellationToken,
) {
    let mut ticker = interval(Duration::from_secs(9));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let _sys_info = rx.borrow_and_update().to_owned();

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
    mut rx: Receiver<SysInfo>,
    cancellation: CancellationToken,
) {
    let mut ticker = interval(Duration::from_secs(9));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let sys_info = rx.borrow_and_update().to_owned();
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
