use std::{collections::HashMap, time::Duration};

use blazing_fan_proto::FanTelemetry;
use opentelemetry::{
    KeyValue, global,
    metrics::{Gauge, MeterProvider},
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{
    Compression, LogExporter, MetricExporter, Protocol, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::{
    Resource,
    logs::{SdkLoggerProvider, log_processor_with_async_runtime::BatchLogProcessor},
    metrics::{SdkMeterProvider, Temporality, periodic_reader_with_async_runtime::PeriodicReader},
    runtime::Tokio,
};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::core::{
    config::OtelConfig,
    port::outbound::otel_port::{OtelError, OtelPort},
    sysinfo::SystemMetrics,
};

struct OtelAdapterInner {
    #[allow(unused)]
    log_provider: SdkLoggerProvider,
    #[allow(unused)]
    meter_provider: SdkMeterProvider,

    cpu_temp_gauge: Gauge<f64>,
    cpu_usage_gauge: Gauge<f64>,
    mem_usage_gauge: Gauge<u64>,
    fan_rpm_gauge: Gauge<u64>,

    attributes: [KeyValue; 1],
}

pub struct OtelAdapter {
    inner: Option<OtelAdapterInner>,
}

impl OtelAdapter {
    pub fn new(config: &OtelConfig, attributes: [KeyValue; 1]) -> color_eyre::Result<Self> {
        if config.enabled {
            let log_provider = OtelAdapter::init_log_provider(config)?;
            let meter_provider = OtelAdapter::init_meter_provider(config)?;
            let meter = meter_provider.meter("blazing-fan-daemon");

            let cpu_tmp_gauge = meter
                .f64_gauge("system.cpu.temperature")
                .with_description("CPU temperature reported by sysinfo component")
                .with_unit("Cel")
                .build();

            let cpu_load_gauge = meter
                .f64_gauge("system.cpu.load")
                .with_description("CPU load reported by sysinfo component")
                .with_unit("Percentage")
                .build();

            let mem_usg_gauge = meter
                .u64_gauge("system.memory.usage")
                .with_description("Memory usage reported by sysinfo component")
                .with_unit("bytes")
                .build();

            let fan_rpm_gauge = meter
                .u64_gauge("fan.rpm")
                .with_description("RPM of the fan")
                .with_unit("rpm")
                .build();

            let inner = Some(OtelAdapterInner {
                log_provider,
                meter_provider,
                cpu_temp_gauge: cpu_tmp_gauge,
                cpu_usage_gauge: cpu_load_gauge,
                mem_usage_gauge: mem_usg_gauge,
                fan_rpm_gauge,
                attributes,
            });

            Ok(Self { inner })
        } else {
            OtelAdapter::init_default_log_provider();

            Ok(Self { inner: None })
        }
    }

    fn init_default_log_provider() {
        let fmt_layer = tracing_subscriber::fmt::layer();

        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,blazing_fan_daemon=debug"));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }

    fn init_log_provider(config: &OtelConfig) -> color_eyre::Result<SdkLoggerProvider> {
        let headers = HashMap::from([
            (
                "authorization".to_string(),
                format!("Bearer {}", config.token),
            ),
            ("dash0-dataset".to_string(), "default".to_string()),
        ]);

        let logs_url = format!("{}/v1/logs", config.endpoint);
        let log_exporter = LogExporter::builder()
            .with_http()
            .with_endpoint(logs_url)
            .with_protocol(Protocol::HttpBinary)
            .with_headers(headers)
            .with_compression(Compression::Zstd)
            .build()?;

        let resource = Resource::builder()
            .with_service_name(config.service_name.clone())
            .build();

        let log_processor = BatchLogProcessor::builder(log_exporter, Tokio).build();

        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_log_processor(log_processor)
            .build();

        let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        let fmt_layer = tracing_subscriber::fmt::layer();

        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,blazing_fan_daemon=debug"));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(otel_log_layer)
            .init();

        Ok(logger_provider)
    }

    fn init_meter_provider(config: &OtelConfig) -> color_eyre::Result<SdkMeterProvider> {
        let headers = HashMap::from([
            (
                "authorization".to_string(),
                format!("Bearer {}", config.token),
            ),
            ("dash0-dataset".to_string(), "default".to_string()),
        ]);

        let metric_url = format!("{}/v1/metrics", config.endpoint);
        let exporter = MetricExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(metric_url)
            .with_headers(headers)
            .with_compression(Compression::Zstd)
            .with_temporality(Temporality::LowMemory)
            .build()?;

        let resource = Resource::builder()
            .with_service_name(config.service_name.clone())
            .build();

        let reader = PeriodicReader::builder(exporter, Tokio).build();

        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();

        global::set_meter_provider(provider.clone());

        Ok(provider)
    }
}

impl OtelPort for OtelAdapter {
    fn record_sys_info(&self, sys_info: &SystemMetrics) {
        if let Some(otel) = self.inner.as_ref() {
            otel.cpu_usage_gauge
                .record(f64::from(sys_info.cpu_usage), &otel.attributes);

            otel.cpu_temp_gauge
                .record(f64::from(sys_info.cpu_tmp), &otel.attributes);

            otel.mem_usage_gauge
                .record(sys_info.mem_usage, &otel.attributes);
        }
    }

    fn recond_fan_telemetry(&self, fan_telemetry: &FanTelemetry) {
        if let Some(otel) = self.inner.as_ref() {
            otel.fan_rpm_gauge
                .record(fan_telemetry.fan_rpm as u64, &otel.attributes);
        }
    }

    fn shutdown(&mut self) -> Result<(), OtelError> {
        if let Some(otel) = self.inner.as_ref() {
            otel.log_provider.shutdown()?;
            otel.meter_provider.shutdown()?;
        }

        Ok(())
    }
}
