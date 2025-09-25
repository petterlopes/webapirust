use std::sync::Arc;

use anyhow::anyhow;
use axum::http::Request;
use governor::middleware::NoOpMiddleware;
use tower_governor::{
    errors::GovernorError, governor::GovernorConfigBuilder, key_extractor::KeyExtractor,
    GovernorLayer,
};

use crate::config::RateLimitConfig;

// Extrator que trata todas as requisições como pertencentes ao mesmo balde.
// Trocar por um extrator por IP (PeerIpKeyExtractor) é fácil caso o ambiente forneça IPs confiáveis.
#[derive(Clone, Default)]
pub struct GlobalKeyExtractor;

impl KeyExtractor for GlobalKeyExtractor {
    type Key = ();

    fn extract<T>(&self, _req: &Request<T>) -> Result<Self::Key, GovernorError> {
        Ok(())
    }
}

pub type RateLimiterLayer = GovernorLayer<GlobalKeyExtractor, NoOpMiddleware>;

// Constrói a camada de rate limit usando os parâmetros definidos em configuração.
pub fn build_rate_limiter(config: &RateLimitConfig) -> anyhow::Result<RateLimiterLayer> {
    let mut builder = GovernorConfigBuilder::default();
    let mut builder = builder.key_extractor(GlobalKeyExtractor::default());

    builder.per_second(config.requests_per_second.max(1));
    builder.burst_size(config.burst_capacity.max(1));

    let cfg = builder
        .finish()
        .ok_or_else(|| anyhow!("invalid rate limiter configuration"))?;

    Ok(GovernorLayer {
        config: Arc::new(cfg),
    })
}
