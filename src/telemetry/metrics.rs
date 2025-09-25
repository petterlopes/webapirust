use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use metrics::counter;

pub type MetricsHandle = PrometheusHandle;
pub type MetricsLayer = PrometheusMetricLayer<'static>;

#[derive(Clone)]
pub struct AppMetrics;

impl AppMetrics {
    const USERS_OP_TOTAL: &'static str = "app_user_operations_total";

    const USER_CREATE_SUCCESS: &'static str = "app_user_create_success_total";
    const USER_CREATE_ERROR: &'static str = "app_user_create_error_total";
    const USER_LIST_SUCCESS: &'static str = "app_user_list_success_total";
    const USER_LIST_ERROR: &'static str = "app_user_list_error_total";
    const USER_GET_SUCCESS: &'static str = "app_user_get_success_total";
    const USER_GET_ERROR: &'static str = "app_user_get_error_total";
    const USER_UPDATE_SUCCESS: &'static str = "app_user_update_success_total";
    const USER_UPDATE_ERROR: &'static str = "app_user_update_error_total";
    const USER_DELETE_SUCCESS: &'static str = "app_user_delete_success_total";
    const USER_DELETE_ERROR: &'static str = "app_user_delete_error_total";

    pub fn new() -> Self {
        Self
    }

    pub fn record_user_operation(
        &self,
        operation: &'static str,
        outcome: &'static str,
        duration: Option<std::time::Duration>,
    ) {
        let _ = duration;
        counter!(Self::USERS_OP_TOTAL);

        if let Some(metric) = Self::counter_metric(operation, outcome) {
            counter!(metric);
        }
    }

    fn counter_metric(operation: &str, outcome: &str) -> Option<&'static str> {
        match (operation, outcome) {
            ("create", "success") => Some(Self::USER_CREATE_SUCCESS),
            ("create", "error") => Some(Self::USER_CREATE_ERROR),
            ("list", "success") => Some(Self::USER_LIST_SUCCESS),
            ("list", "error") => Some(Self::USER_LIST_ERROR),
            ("get", "success") => Some(Self::USER_GET_SUCCESS),
            ("get", "error") => Some(Self::USER_GET_ERROR),
            ("update", "success") => Some(Self::USER_UPDATE_SUCCESS),
            ("update", "error") => Some(Self::USER_UPDATE_ERROR),
            ("delete", "success") => Some(Self::USER_DELETE_SUCCESS),
            ("delete", "error") => Some(Self::USER_DELETE_ERROR),
            _ => None,
        }
    }
}

pub fn init_metrics() -> (MetricsLayer, MetricsHandle, AppMetrics) {
    let (layer, handle) = PrometheusMetricLayer::pair();
    let metrics = AppMetrics::new();
    (layer, handle, metrics)
}
