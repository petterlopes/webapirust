use std::sync::Arc;

use anyhow::{ensure, Context};
use tokio::net::TcpListener;

use webrust::app::{build_rate_limiter, build_router, AppState};
use webrust::application::services::auth_service::AuthService;
use webrust::application::services::user_service::UserService;
use webrust::config;
use webrust::domain::repositories::user_repository::UserRepository;
use webrust::infrastructure::database;
use webrust::infrastructure::repositories::postgres_user_repository::PostgresUserRepository;
use webrust::shared::security::token::JwtManager;
use webrust::telemetry::{init_metrics, init_tracing, AuditLogger};

// The entry point wires together configuration, observability, persistence and the Axum router.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load settings from YAML + environment, fail fast if algo estiver inconsistente.
    let configuration = config::load().context("failed to load application configuration")?;

    ensure!(
        configuration.auth.jwt_ttl_minutes > 0,
        "auth.jwt_ttl_minutes must be greater than zero"
    );

    // Tracing precisa ser iniciado antes de qualquer log para capturar boot e diagnÃ³sticos.
    init_tracing(
        &configuration.telemetry.service_name,
        &configuration.telemetry.log_level,
    )
    .context("failed to initialise tracing")?;

    // Camada de mÃ©tricas + handle Prometheus e rate limiting sÃ£o construÃ­dos antes da aplicaÃ§Ã£o.
    let (metrics_layer, metrics_handle, app_metrics) = init_metrics();
    let rate_limiter_layer = build_rate_limiter(&configuration.rate_limit)?;

    // Conecta ao Postgres e garante que o pool esteja pronto para receber requisiÃ§Ãµes.
    let pool = database::init_pool(&configuration.database)
        .await
        .context("failed to initialise database connection pool")?;

    // Executa migraÃ§Ãµes pendentes; idealmente rodaria tambÃ©m em pipeline CI.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    // Ainda ganhamos flexibilidade usando trait objects: Ã© fÃ¡cil trocar o repositÃ³rio por outro backend.
    let repository: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_service = UserService::new(repository.clone());
    let jwt_manager = JwtManager::new(
        &configuration.auth.jwt_secret,
        configuration.auth.jwt_ttl_minutes,
    );
    let auth_service = AuthService::new(repository, jwt_manager);

    if configuration.bootstrap.enabled {
        match user_service
            .ensure_admin_account(
                &configuration.bootstrap.admin_name,
                &configuration.bootstrap.admin_email,
                &configuration.bootstrap.admin_password,
            )
            .await
        {
            Ok(true) => tracing::info!(
                email = %configuration.bootstrap.admin_email,
                "bootstrap admin created"
            ),
            Ok(false) => tracing::info!(
                email = %configuration.bootstrap.admin_email,
                "bootstrap admin already present"
            ),
            Err(err) => {
                return Err(anyhow::anyhow!(
                    "failed to ensure bootstrap admin account: {}",
                    err
                ));
            }
        }
    }

    // O estado compartilhado carrega os serviÃ§os de domÃ­nio e ganchos de telemetria.
    let audit_logger = AuditLogger::new();
    let state = AppState::new(
        user_service,
        auth_service,
        metrics_handle,
        app_metrics,
        audit_logger,
    );
    let router = build_router(state, metrics_layer, rate_limiter_layer);

    // Subimos o listener TCP e logamos o endereÃ§o final.
    let address = configuration.address();
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("failed to bind to {address}"))?;

    tracing::info!(%address, "server started");

    // Axum assume o controle do loop de requisiÃ§Ãµes; qualquer erro encerra o processo com contexto.
    axum::serve(listener, router.into_make_service())
        .await
        .context("server error")
}
