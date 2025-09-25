use std::ops::Deref;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::{HeaderValue, AUTHORIZATION};
use axum::http::request::Parts;

use crate::app::AppState;
use crate::application::services::auth_service::AuthenticatedUser;
use crate::shared::error::AppError;

const BEARER_PREFIX: &str = "Bearer ";

pub struct CurrentUser(pub AuthenticatedUser);

impl Deref for CurrentUser {
    type Target = AuthenticatedUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CurrentUser {
    pub fn into_inner(self) -> AuthenticatedUser {
        self.0
    }
}

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".to_string()))?;

        let token = extract_bearer_token(header)?;
        let user = state.auth_service().verify(token)?;

        Ok(CurrentUser(user))
    }
}

fn extract_bearer_token(value: &HeaderValue) -> Result<&str, AppError> {
    let raw = value
        .to_str()
        .map_err(|_| AppError::Unauthorized("invalid authorization header".to_string()))?;

    if !raw.starts_with(BEARER_PREFIX) {
        return Err(AppError::Unauthorized(
            "authorization header must be a Bearer token".to_string(),
        ));
    }

    let token = raw.trim_start_matches(BEARER_PREFIX).trim();

    if token.is_empty() {
        return Err(AppError::Unauthorized(
            "authorization header must not be empty".to_string(),
        ));
    }

    Ok(token)
}
