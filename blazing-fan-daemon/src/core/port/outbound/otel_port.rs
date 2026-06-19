use crate::core::sysinfo::SystemMetrics;
use blazing_fan_proto::FanTelemetry;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OtelError {
    #[error("otel sdk error {0}")]
    OtelSdk(#[from] opentelemetry_sdk::error::OTelSdkError),
}

pub trait OtelPort {
    fn record_sys_info(&self, sys_info: &SystemMetrics);

    fn recond_fan_telemetry(&self, fan_telemetry: &FanTelemetry);

    fn shutdown(&mut self) -> Result<(), OtelError>;
}
