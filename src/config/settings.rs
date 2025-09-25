use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub telemetry: TelemetryConfig,
    pub rate_limit: RateLimitConfig,
    pub auth: AuthConfig,
    pub bootstrap: BootstrapConfig,
}

impl AppConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub max_connections: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub log_level: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u64,
    pub burst_capacity: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_ttl_minutes: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BootstrapConfig {
    pub enabled: bool,
    pub admin_name: String,
    pub admin_email: String,
    pub admin_password: String,
}
