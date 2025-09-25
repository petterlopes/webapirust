use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::user::{NewUser, UpdateUser, User};
use crate::shared::error::AppError;

pub type RepositoryResult<T> = Result<T, AppError>;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, new_user: NewUser) -> RepositoryResult<User>;
    async fn find_all(&self) -> RepositoryResult<Vec<User>>;
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<User>>;
    async fn update(&self, id: Uuid, update: UpdateUser) -> RepositoryResult<User>;
    async fn delete(&self, id: Uuid) -> RepositoryResult<()>;
}
