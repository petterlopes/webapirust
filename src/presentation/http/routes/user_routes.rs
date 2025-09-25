use axum::{routing::get, routing::post, Router};

use crate::app::AppState;
use crate::presentation::http::controllers::users_controller;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/users",
            post(users_controller::create_user).get(users_controller::list_users),
        )
        .route(
            "/users/:id",
            get(users_controller::get_user)
                .put(users_controller::update_user)
                .delete(users_controller::delete_user),
        )
}
