use std::str::FromStr;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::domain::entities::user::{NewUser, UpdateUser, User, UserRole};
use crate::domain::errors::DomainError;
use crate::domain::repositories::user_repository::{RepositoryResult, UserRepository};
use crate::shared::error::AppError;

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[derive(Debug, Clone, FromRow)]
struct UserRecord {
    id: Uuid,
    name: String,
    email: String,
    password_hash: String,
    role: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRecord> for User {
    type Error = AppError;

    fn try_from(record: UserRecord) -> Result<Self, Self::Error> {
        let role = UserRole::from_str(&record.role).map_err(|err| {
            AppError::Unexpected(anyhow!("failed to parse persisted role: {}", err))
        })?;

        User::try_new(
            record.id,
            &record.name,
            &record.email,
            role,
            &record.password_hash,
            record.created_at,
            record.updated_at,
        )
        .map_err(map_domain_error)
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, new_user: NewUser) -> RepositoryResult<User> {
        let id = Uuid::new_v4();

        let record = sqlx::query_as::<_, UserRecord>(
            "INSERT INTO users (id, name, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, email, password_hash, role, created_at, updated_at",
        )
        .bind(id)
        .bind(new_user.name().as_str())
        .bind(new_user.email().as_str())
        .bind(new_user.password_hash().as_str())
        .bind(new_user.role().as_str())
        .fetch_one(self.pool())
        .await?;

        record.try_into()
    }

    async fn find_all(&self) -> RepositoryResult<Vec<User>> {
        let records = sqlx::query_as::<_, UserRecord>(
            "SELECT id, name, email, password_hash, role, created_at, updated_at
             FROM users
             ORDER BY created_at DESC",
        )
        .fetch_all(self.pool())
        .await?;

        records.into_iter().map(TryInto::try_into).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<User>> {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT id, name, email, password_hash, role, created_at, updated_at
             FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;

        match record {
            Some(record) => Ok(Some(record.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<User>> {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT id, name, email, password_hash, role, created_at, updated_at
             FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(self.pool())
        .await?;

        match record {
            Some(record) => Ok(Some(record.try_into()?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: Uuid, update: UpdateUser) -> RepositoryResult<User> {
        let record = sqlx::query_as::<_, UserRecord>(
            "UPDATE users
             SET name = COALESCE($2, name),
                 email = COALESCE($3, email),
                 password_hash = COALESCE($4, password_hash),
                 role = COALESCE($5, role),
                 updated_at = NOW()
             WHERE id = $1
             RETURNING id, name, email, password_hash, role, created_at, updated_at",
        )
        .bind(id)
        .bind(update.name_str())
        .bind(update.email_str())
        .bind(update.password_hash_str())
        .bind(update.role().map(|role| role.as_str().to_string()))
        .fetch_optional(self.pool())
        .await?;

        match record {
            Some(record) => record.try_into(),
            None => Err(AppError::NotFound(format!("user {id} not found"))),
        }
    }

    async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("user {id} not found")));
        }

        Ok(())
    }
}

fn map_domain_error(error: DomainError) -> AppError {
    match error {
        DomainError::Validation(message) => AppError::Validation(message),
    }
}
