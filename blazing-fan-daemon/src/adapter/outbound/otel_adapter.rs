use std::collections::HashMap;

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
    sysinfo::BladeTelemetry,
};

struct OtelAdapterInner {
    logger: SdkLoggerProvider,
    meter: SdkMeterProvider,

    f64_gauges: HashMap<String, Gauge<f64>>,
    u64_gauges: HashMap<String, Gauge<u64>>,

    attributes: [KeyValue; 1],
}

pub struct OtelAdapter {
    inner: Option<OtelAdapterInner>,
}

impl OtelAdapter {
    pub fn new(config: &OtelConfig, attributes: [KeyValue; 1]) -> color_eyre::Result<Self> {
        if config.enabled {
            let logger = OtelAdapter::init_log_provider(config)?;
            let meter = OtelAdapter::init_meter_provider(config)?;
            let inner = Some(OtelAdapterInner {
                logger,
                meter,
                f64_gauges: HashMap::new(),
                u64_gauges: HashMap::new(),
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

    fn record_u64_metric(&mut self, name: String, value: u64, description: String, unit: String) {
        if let Some(otel) = self.inner.as_mut() {
            match otel.u64_gauges.get(&name) {
                Some(gauge) => gauge.record(value, &otel.attributes),
                None => {
                    let gauge = otel
                        .meter
                        .meter("blazing-fan-daemon")
                        .u64_gauge(name.clone())
                        .with_description(description)
                        .with_unit(unit)
                        .build();

                    otel.u64_gauges.insert(name, gauge);
                }
            }
        }
    }

    fn record_f64_metric(&mut self, name: String, value: f64, description: String, unit: String) {
        if let Some(otel) = self.inner.as_mut() {
            match otel.f64_gauges.get(&name) {
                Some(gauge) => gauge.record(value, &otel.attributes),
                None => {
                    let gauge = otel
                        .meter
                        .meter("blazing-fan-daemon")
                        .f64_gauge(name.clone())
                        .with_description(description)
                        .with_unit(unit)
                        .build();

                    otel.f64_gauges.insert(name, gauge);
                }
            }
        }
    }
}

impl OtelPort for OtelAdapter {
    fn record_blade_telemetry(&mut self, sys_info: &BladeTelemetry) {
        self.record_f64_metric(
            String::from("system.cpu.usage"),
            f64::from(sys_info.cpu.usage),
            String::from("system cpu usage"),
            String::from("percentage"),
        );

        self.record_f64_metric(
            String::from("system.cpu.temp"),
            f64::from(sys_info.cpu.temp),
            String::from("system cpu temp"),
            String::from("celcius"),
        );

        self.record_u64_metric(
            String::from("system.memory.ram.usage"),
            sys_info.memory.ram_usage,
            String::from("system memory ram usage"),
            String::from("bytes"),
        );

        self.record_u64_metric(
            String::from("system.memory.swap.usage"),
            sys_info.memory.swap_usage,
            String::from("system memory swap usage"),
            String::from("bytes"),
        );

        for disk in sys_info.disks.iter() {
            let name = disk.name.clone();
            self.record_u64_metric(
                format!("system.disk.{name}.available"),
                disk.available,
                String::from("system disk usage"),
                String::from("bytes"),
            );
        }

        for interface in sys_info.networks.iter() {
            let name = interface.name.clone();
            self.record_u64_metric(
                format!("system.interface.{name}.received"),
                interface.received,
                String::from("system network interface received"),
                String::from("bytes"),
            );
            self.record_u64_metric(
                format!("system.interface.{name}.transmitted"),
                interface.transmitted,
                String::from("system network interface transmitted"),
                String::from("bytes"),
            );
        }
    }

    fn recond_fan_telemetry(&mut self, fan_telemetry: &FanTelemetry) {
        self.record_u64_metric(
            String::from("fan.rpm"),
            fan_telemetry.fan_rpm as u64,
            String::from("fan rpm"),
            String::from("rpm"),
        );
    }

    fn shutdown(&mut self) -> Result<(), OtelError> {
        if let Some(otel) = self.inner.as_ref() {
            otel.logger.shutdown()?;
            otel.meter.shutdown()?;
        }

        Ok(())
    }
}
