use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::services::auth_service::AuthSession;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequestDto {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct LoginResponseDto {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub user: AuthenticatedUserDto,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct AuthenticatedUserDto {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl From<AuthSession> for LoginResponseDto {
    fn from(session: AuthSession) -> Self {
        Self {
            access_token: session.token,
            expires_at: session.expires_at,
            user: AuthenticatedUserDto {
                id: session.user.id,
                email: session.user.email,
                role: session.user.role.as_str().to_string(),
            },
        }
    }
}
