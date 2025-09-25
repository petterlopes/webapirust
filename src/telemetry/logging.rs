use anyhow::anyhow;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(service_name: &str, default_level: &str) -> anyhow::Result<()> {
    let env_filter = std::env::var("RUST_LOG")
        .ok()
        .and_then(|_| EnvFilter::try_from_default_env().ok())
        .unwrap_or_else(|| EnvFilter::new(default_level));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|err| anyhow!("failed to initialize tracing subscriber: {err}"))?;

    tracing::info!(service.name = service_name, "tracing initialized");
    Ok(())
}
