use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub uart: UartConfig,
    pub polling: PollingConfig,
    pub otel: OtelConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UartConfig {
    pub path: String,
    pub baud_rate: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollingConfig {
    pub interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OtelConfig {
    pub enabled: bool,
    pub service_name: String,
    pub endpoint: String,
    pub token: String,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let dirs = ProjectDirs::from("dev", "thatwhichis", "blazing-fan-daemon")
        .ok_or_else(|| ConfigError::NotFound("app config dir doesn't exist".to_string()))?;

    let config_path = dirs.config_dir().join("config.toml");
    let config = Config::builder()
        .add_source(File::from(config_path.as_path()).required(true))
        .build()?;

    let app_config: AppConfig = config.try_deserialize()?;

    Ok(app_config)
}
