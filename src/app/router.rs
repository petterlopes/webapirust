use axum::{extract::State, http::StatusCode, response::Response, routing::get, Router};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::{AppState, RateLimiterLayer};
use crate::presentation::http::{docs::ApiDoc, routes};
use crate::shared::validation;
use crate::telemetry::MetricsLayer;

// Responsavel por montar o grafo de rotas, empilhando middlewares de CORS,
// rate limit, tracing e metricas.
pub fn build_router(
    state: AppState,
    metrics_layer: MetricsLayer,
    rate_limiter_layer: RateLimiterLayer,
) -> Router {
    let openapi = ApiDoc::openapi();
    let swagger_ui = SwaggerUi::new("/docs").url("/docs/openapi.json", openapi);

    Router::new()
        .merge(routes::auth_routes())
        .merge(routes::user_routes())
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .merge(swagger_ui)
        .layer(RequestBodyLimitLayer::new(validation::MAX_JSON_BODY_BYTES))
        .layer(CorsLayer::permissive())
        .layer(rate_limiter_layer)
        .layer(metrics_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "ok"
}

async fn metrics_handler(State(state): State<AppState>) -> Response {
    let mut body = state.metrics_handle().render();

    // O exporter prefixa com quebras de linha; removemos para manter o formato aceito pelo Prometheus.
    while body.starts_with('\n') || body.starts_with('\r') {
        body.remove(0);
    }

    if !body.ends_with('\n') {
        body.push('\n');
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )
        .body(axum::body::Body::from(body))
        .expect("failed to build metrics response")
}
