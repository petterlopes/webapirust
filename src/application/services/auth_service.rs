use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::user::UserRole;
use crate::domain::repositories::user_repository::UserRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::security::password::PasswordError;
use crate::shared::security::{
    password,
    token::{Claims, JwtManager, TokenError},
};

#[derive(Clone)]
pub struct AuthService {
    repository: Arc<dyn UserRepository>,
    jwt: JwtManager,
}

impl AuthService {
    pub fn new(repository: Arc<dyn UserRepository>, jwt: JwtManager) -> Self {
        Self { repository, jwt }
    }

    pub async fn authenticate(&self, email: &str, password_input: &str) -> AppResult<AuthSession> {
        let user = self
            .repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid credentials".to_string()))?;

        password::verify_password(user.password_hash().as_str(), password_input).map_err(
            |err| match err {
                PasswordError::InvalidPassword => {
                    AppError::Unauthorized("invalid credentials".to_string())
                }
                PasswordError::Hash(_) => {
                    AppError::Unexpected(anyhow!("failed to verify stored password hash"))
                }
            },
        )?;

        let token = self
            .jwt
            .generate(user.id(), user.email().as_str(), user.role().as_str())
            .map_err(|err| AppError::Unexpected(anyhow!("failed to issue token: {err}")))?;

        Ok(AuthSession {
            token: token.token,
            expires_at: token.expires_at,
            user: AuthenticatedUser {
                id: user.id(),
                email: user.email().as_str().to_string(),
                role: user.role(),
            },
        })
    }

    pub fn verify(&self, token: &str) -> AppResult<AuthenticatedUser> {
        let claims = self.jwt.verify(token).map_err(map_token_error)?;

        claims.try_into()
    }
}

fn map_token_error(err: TokenError) -> AppError {
    match err {
        TokenError::Expired => AppError::Unauthorized("token expired".to_string()),
        TokenError::Invalid(_) => AppError::Unauthorized("invalid token".to_string()),
        TokenError::InvalidTtl => AppError::Unexpected(anyhow!("token generated with invalid ttl")),
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

impl TryFrom<Claims> for AuthenticatedUser {
    type Error = AppError;

    fn try_from(value: Claims) -> Result<Self, Self::Error> {
        let role = UserRole::from_str(&value.role)
            .map_err(|err| AppError::Unexpected(anyhow!("invalid role in token: {err}")))?;

        Ok(Self {
            id: value.sub,
            email: value.email,
            role,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: AuthenticatedUser,
}

impl AuthenticatedUser {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn role(&self) -> &UserRole {
        &self.role
    }
}
