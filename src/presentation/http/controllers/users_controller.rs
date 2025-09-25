use std::time::Instant;

use axum::{extract::Path, extract::State, http::StatusCode, Json};
use uuid::Uuid;

use crate::app::AppState;
use crate::application::dtos::user::{CreateUserDto, UpdateUserDto, UserResponseDto};
use crate::application::services::auth_service::AuthenticatedUser;
use crate::presentation::http::auth::extractor::CurrentUser;
#[allow(unused_imports)]
use crate::shared::error::{AppError, AppResult, ErrorResponse};
use crate::shared::validation::sanitize_for_logging;
use crate::telemetry::{AuditActor, AuditEvent, AuditTarget};

const OP_CREATE: &str = "create";
const OP_LIST: &str = "list";
const OP_GET: &str = "get";
const OP_UPDATE: &str = "update";
const OP_DELETE: &str = "delete";
const OUTCOME_SUCCESS: &str = "success";
const OUTCOME_ERROR: &str = "error";

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created", body = UserResponseDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 409, description = "Conflict", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    security(("bearerAuth" = [])),
    tag = "Users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserResponseDto>), AppError> {
    let actor = audit_actor(&current_user);
    let started = Instant::now();

    match state
        .user_service()
        .create_user(&current_user, payload)
        .await
    {
        Ok(user) => {
            state.metrics().record_user_operation(
                OP_CREATE,
                OUTCOME_SUCCESS,
                Some(started.elapsed()),
            );
            let detail = sanitize_for_logging(&format!("created user {}", user.email()));
            state.audit().log(AuditEvent::success(
                "user.create",
                actor,
                AuditTarget::new("user", Some(user.id().to_string())),
                Some(detail),
                None,
            ));
            Ok((StatusCode::CREATED, Json(user)))
        }
        Err(err) => {
            state.metrics().record_user_operation(
                OP_CREATE,
                OUTCOME_ERROR,
                Some(started.elapsed()),
            );
            let detail = sanitize_for_logging(&err.to_string());
            state.audit().log(AuditEvent::failure(
                "user.create",
                actor,
                AuditTarget::new("user", None),
                Some(detail),
                None,
            ));
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List users", body = [UserResponseDto]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    security(("bearerAuth" = [])),
    tag = "Users"
)]
pub async fn list_users(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
) -> Result<Json<Vec<UserResponseDto>>, AppError> {
    let started = Instant::now();

    match state.user_service().list_users(&current_user).await {
        Ok(users) => {
            state.metrics().record_user_operation(
                OP_LIST,
                OUTCOME_SUCCESS,
                Some(started.elapsed()),
            );
            Ok(Json(users))
        }
        Err(err) => {
            state
                .metrics()
                .record_user_operation(OP_LIST, OUTCOME_ERROR, Some(started.elapsed()));
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(("id" = uuid::Uuid, Path, description = "user.id()entifier")),
    responses(
        (status = 200, description = "User detail", body = UserResponseDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    security(("bearerAuth" = [])),
    tag = "Users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponseDto>, AppError> {
    let started = Instant::now();

    match state.user_service().get_user(&current_user, id).await {
        Ok(user) => {
            state
                .metrics()
                .record_user_operation(OP_GET, OUTCOME_SUCCESS, Some(started.elapsed()));
            Ok(Json(user))
        }
        Err(err) => {
            state
                .metrics()
                .record_user_operation(OP_GET, OUTCOME_ERROR, Some(started.elapsed()));
            Err(err)
        }
    }
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserDto,
    params(("id" = uuid::Uuid, Path, description = "user.id()entifier")),
    responses(
        (status = 200, description = "User updated", body = UserResponseDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    security(("bearerAuth" = [])),
    tag = "Users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<UserResponseDto>, AppError> {
    let actor = audit_actor(&current_user);
    let started = Instant::now();

    match state
        .user_service()
        .update_user(&current_user, id, payload)
        .await
    {
        Ok(user) => {
            state.metrics().record_user_operation(
                OP_UPDATE,
                OUTCOME_SUCCESS,
                Some(started.elapsed()),
            );
            state.audit().log(AuditEvent::success(
                "user.update",
                actor,
                AuditTarget::new("user", Some(user.id().to_string())),
                None,
                None,
            ));
            Ok(Json(user))
        }
        Err(err) => {
            state.metrics().record_user_operation(
                OP_UPDATE,
                OUTCOME_ERROR,
                Some(started.elapsed()),
            );
            let detail = sanitize_for_logging(&err.to_string());
            state.audit().log(AuditEvent::failure(
                "user.update",
                actor,
                AuditTarget::new("user", Some(id.to_string())),
                Some(detail),
                None,
            ));
            Err(err)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = uuid::Uuid, Path, description = "user.id()entifier")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    security(("bearerAuth" = [])),
    tag = "Users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let actor = audit_actor(&current_user);
    let started = Instant::now();

    match state.user_service().delete_user(&current_user, id).await {
        Ok(()) => {
            state.metrics().record_user_operation(
                OP_DELETE,
                OUTCOME_SUCCESS,
                Some(started.elapsed()),
            );
            state.audit().log(AuditEvent::success(
                "user.delete",
                actor,
                AuditTarget::new("user", Some(id.to_string())),
                None,
                None,
            ));
            Ok(StatusCode::NO_CONTENT)
        }
        Err(err) => {
            state.metrics().record_user_operation(
                OP_DELETE,
                OUTCOME_ERROR,
                Some(started.elapsed()),
            );
            let detail = sanitize_for_logging(&err.to_string());
            state.audit().log(AuditEvent::failure(
                "user.delete",
                actor,
                AuditTarget::new("user", Some(id.to_string())),
                Some(detail),
                None,
            ));
            Err(err)
        }
    }
}

fn audit_actor(user: &AuthenticatedUser) -> AuditActor {
    AuditActor {
        id: Some(user.id()),
        email: Some(sanitize_for_logging(user.email())),
        role: Some(user.role().as_str().to_string()),
    }
}
