use crate::core::sysinfo::SysInfo;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OtelError {
    #[error("otel sdk error {0}")]
    OtelSdk(#[from] opentelemetry_sdk::error::OTelSdkError),
}

pub trait OtelPort {
    fn record_sys_info(&self, sys_info: &SysInfo);

    fn shutdown(&mut self) -> Result<(), OtelError>;
}
