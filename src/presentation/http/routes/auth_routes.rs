use axum::routing::post;
use axum::Router;

use crate::app::AppState;
use crate::presentation::http::controllers::auth_controller;

pub fn auth_routes() -> Router<AppState> {
    Router::new().route("/auth/login", post(auth_controller::login))
}
