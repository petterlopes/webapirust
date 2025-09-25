use crate::application::services::auth_service::AuthService;
use crate::application::services::user_service::UserService;
use crate::telemetry::{AppMetrics, AuditLogger, MetricsHandle};

#[derive(Clone)]
pub struct AppState {
    user_service: UserService,
    auth_service: AuthService,
    metrics_handle: MetricsHandle,
    app_metrics: AppMetrics,
    audit_logger: AuditLogger,
}

impl AppState {
    pub fn new(
        user_service: UserService,
        auth_service: AuthService,
        metrics_handle: MetricsHandle,
        app_metrics: AppMetrics,
        audit_logger: AuditLogger,
    ) -> Self {
        Self {
            user_service,
            auth_service,
            metrics_handle,
            app_metrics,
            audit_logger,
        }
    }

    pub fn user_service(&self) -> &UserService {
        &self.user_service
    }

    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    pub fn metrics_handle(&self) -> &MetricsHandle {
        &self.metrics_handle
    }

    pub fn metrics(&self) -> &AppMetrics {
        &self.app_metrics
    }

    pub fn audit(&self) -> &AuditLogger {
        &self.audit_logger
    }
}
