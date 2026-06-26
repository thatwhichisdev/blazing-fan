use crate::core::sysinfo::BladeTelemetry;
use blazing_fan_proto::FanTelemetry;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OtelError {
    #[error("otel sdk error {0}")]
    OtelSdk(#[from] opentelemetry_sdk::error::OTelSdkError),
}

pub trait OtelPort {
    fn record_blade_telemetry(&mut self, sys_info: &BladeTelemetry);

    fn recond_fan_telemetry(&mut self, fan_telemetry: &FanTelemetry);

    fn shutdown(&mut self) -> Result<(), OtelError>;
}
