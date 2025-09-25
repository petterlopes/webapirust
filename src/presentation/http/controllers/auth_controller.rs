use axum::{extract::State, Json};

use crate::app::AppState;
use crate::application::dtos::auth::{LoginRequestDto, LoginResponseDto};
#[allow(unused_imports)]
use crate::shared::error::{AppResult, ErrorResponse};
use crate::shared::validation::sanitize_for_logging;
use crate::telemetry::{AuditActor, AuditEvent, AuditTarget};

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequestDto,
    responses(
        (status = 200, description = "Authenticated successfully", body = LoginResponseDto),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequestDto>,
) -> AppResult<Json<LoginResponseDto>> {
    let email = payload.email.clone();

    match state
        .auth_service()
        .authenticate(&payload.email, &payload.password)
        .await
    {
        Ok(session) => {
            let actor = AuditActor {
                id: Some(session.user.id),
                email: Some(sanitize_for_logging(&session.user.email)),
                role: Some(session.user.role.as_str().to_string()),
            };

            state.audit().log(AuditEvent::success(
                "auth.login",
                actor,
                AuditTarget::new("auth", Some(session.user.id.to_string())),
                None,
                None,
            ));

            Ok(Json(LoginResponseDto::from(session)))
        }
        Err(err) => {
            state.audit().log(AuditEvent::failure(
                "auth.login",
                AuditActor {
                    id: None,
                    email: Some(sanitize_for_logging(&email)),
                    role: None,
                },
                AuditTarget::new("auth", None),
                Some(sanitize_for_logging(&err.to_string())),
                None,
            ));

            Err(err)
        }
    }
}
