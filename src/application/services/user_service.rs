use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use uuid::Uuid;

use crate::application::dtos::user::{CreateUserDto, UpdateUserDto, UserResponseDto};
use crate::application::services::auth_service::AuthenticatedUser;
use crate::domain::entities::user::{NewUser, UpdateUser, UserRole};
use crate::domain::errors::DomainError;
use crate::domain::repositories::user_repository::UserRepository;
use crate::domain::value_objects::{EmailAddress, PasswordHash, PlainPassword, UserName};
use crate::shared::error::{AppError, AppResult};
use crate::shared::security::password;

#[derive(Clone)]
pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_user(
        &self,
        actor: &AuthenticatedUser,
        dto: CreateUserDto,
    ) -> AppResult<UserResponseDto> {
        ensure_admin(actor)?;
        self.create_user_internal(dto).await
    }

    pub async fn list_users(&self, actor: &AuthenticatedUser) -> AppResult<Vec<UserResponseDto>> {
        if actor.role == UserRole::Admin {
            let users = self.repository.find_all().await?;
            return Ok(users.into_iter().map(Into::into).collect());
        }

        match self.repository.find_by_id(actor.id).await? {
            Some(user) => Ok(vec![user.into()]),
            None => Err(AppError::NotFound(format!("user {} not found", actor.id))),
        }
    }

    pub async fn get_user(
        &self,
        actor: &AuthenticatedUser,
        id: Uuid,
    ) -> AppResult<UserResponseDto> {
        if actor.role != UserRole::Admin && actor.id != id {
            return Err(AppError::Forbidden("insufficient privileges".to_string()));
        }

        match self.repository.find_by_id(id).await? {
            Some(user) => Ok(user.into()),
            None => Err(AppError::NotFound(format!("user {id} not found"))),
        }
    }

    pub async fn update_user(
        &self,
        actor: &AuthenticatedUser,
        id: Uuid,
        dto: UpdateUserDto,
    ) -> AppResult<UserResponseDto> {
        ensure_admin(actor)?;
        self.update_user_internal(id, dto).await
    }

    pub async fn delete_user(&self, actor: &AuthenticatedUser, id: Uuid) -> AppResult<()> {
        ensure_admin(actor)?;
        self.repository.delete(id).await
    }

    pub async fn ensure_admin_account(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> AppResult<bool> {
        if self.repository.find_by_email(email).await?.is_some() {
            return Ok(false);
        }

        let dto = CreateUserDto {
            name: name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            role: "admin".to_string(),
        };

        match self.create_user_internal(dto).await {
            Ok(_) => Ok(true),
            Err(AppError::Conflict(_)) => Ok(false),
            Err(err) => Err(err),
        }
    }

    async fn create_user_internal(&self, dto: CreateUserDto) -> AppResult<UserResponseDto> {
        let CreateUserDto {
            name,
            email,
            password,
            role,
        } = dto;

        let role = parse_role(&role)?;
        let user_name = UserName::parse(&name).map_err(map_domain_error)?;
        let email_address = EmailAddress::parse(&email).map_err(map_domain_error)?;
        let plain_password = PlainPassword::parse(&password).map_err(map_domain_error)?;
        let password_hash_raw = password::hash_password(plain_password.as_str())
            .map_err(|err| AppError::Unexpected(anyhow!("failed to hash password: {err}")))?;
        let password_hash = PasswordHash::new(&password_hash_raw).map_err(map_domain_error)?;

        let new_user = NewUser::build(user_name, email_address, password_hash, role);
        let user = self.repository.create(new_user).await?;
        Ok(user.into())
    }

    async fn update_user_internal(
        &self,
        id: Uuid,
        dto: UpdateUserDto,
    ) -> AppResult<UserResponseDto> {
        let mut update = UpdateUser::default();

        if let Some(name) = dto.name {
            let parsed = UserName::parse(&name).map_err(map_domain_error)?;
            update = update.apply_name(parsed);
        }

        if let Some(email) = dto.email {
            let parsed = EmailAddress::parse(&email).map_err(map_domain_error)?;
            update = update.apply_email(parsed);
        }

        if let Some(password) = dto.password {
            let plain_password = PlainPassword::parse(&password).map_err(map_domain_error)?;
            let password_hash_raw = password::hash_password(plain_password.as_str())
                .map_err(|err| AppError::Unexpected(anyhow!("failed to hash password: {err}")))?;
            let password_hash = PasswordHash::new(&password_hash_raw).map_err(map_domain_error)?;
            update = update.apply_password_hash(password_hash);
        }

        if let Some(role) = dto.role {
            update = update.apply_role(parse_role(&role)?);
        }

        if update.is_empty() {
            return Err(AppError::Validation(
                "at least one field must be provided".to_string(),
            ));
        }

        let user = self.repository.update(id, update).await?;
        Ok(user.into())
    }
}

fn parse_role(raw: &str) -> AppResult<UserRole> {
    let normalized = raw.trim().to_lowercase();

    UserRole::from_str(&normalized).map_err(|err| AppError::Validation(err.to_string()))
}

fn ensure_admin(actor: &AuthenticatedUser) -> AppResult<()> {
    if actor.role == UserRole::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin role required".to_string()))
    }
}

fn map_domain_error(error: DomainError) -> AppError {
    match error {
        DomainError::Validation(message) => AppError::Validation(message),
    }
}
