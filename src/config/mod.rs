mod settings;

pub use settings::{
    AppConfig, AuthConfig, BootstrapConfig, DatabaseConfig, RateLimitConfig, ServerConfig,
    TelemetryConfig,
};

use anyhow::Context;
use config as config_crate;

const DEFAULT_CONFIG_FILE: &str = "configuration/default";
const LOCAL_CONFIG_FILE: &str = "configuration/local";
const ENV_PREFIX: &str = "APP";

pub fn load() -> anyhow::Result<AppConfig> {
    let _ = dotenvy::dotenv();

    let builder = config_crate::Config::builder()
        .add_source(config_crate::File::with_name(DEFAULT_CONFIG_FILE))
        .add_source(config_crate::File::with_name(LOCAL_CONFIG_FILE).required(false))
        .add_source(config_crate::Environment::with_prefix(ENV_PREFIX).separator("__"));

    builder
        .build()
        .context("failed to build application configuration")?
        .try_deserialize::<AppConfig>()
        .context("failed to deserialize configuration into AppConfig")
}
