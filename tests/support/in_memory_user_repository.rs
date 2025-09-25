use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use webrust::domain::entities::user::{NewUser, UpdateUser, User};
use webrust::domain::repositories::user_repository::{RepositoryResult, UserRepository};
use webrust::shared::error::AppError;

#[derive(Clone, Default)]
pub struct InMemoryUserRepository {
    store: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }

    async fn email_exists(&self, email: &str, ignore_id: Option<Uuid>) -> bool {
        let store = self.store.read().await;
        store
            .values()
            .filter(|user| Some(user.id()) != ignore_id)
            .any(|user| user.email().as_str().eq_ignore_ascii_case(email))
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, new_user: NewUser) -> RepositoryResult<User> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let user = User::new(
            id,
            new_user.name().clone(),
            new_user.email().clone(),
            new_user.role(),
            new_user.password_hash().clone(),
            now,
            now,
        );

        if self.email_exists(user.email().as_str(), None).await {
            return Err(AppError::Conflict(format!(
                "user {} already exists",
                user.email().as_str()
            )));
        }

        let mut store = self.store.write().await;
        store.insert(id, user.clone());
        Ok(user)
    }

    async fn find_all(&self) -> RepositoryResult<Vec<User>> {
        let store = self.store.read().await;
        let mut users: Vec<User> = store.values().cloned().collect();
        users.sort_by_key(|user| user.created_at());
        users.reverse();
        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<User>> {
        let store = self.store.read().await;
        Ok(store.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<User>> {
        let store = self.store.read().await;
        Ok(store
            .values()
            .find(|user| user.email().as_str().eq_ignore_ascii_case(email))
            .cloned())
    }

    async fn update(&self, id: Uuid, update: UpdateUser) -> RepositoryResult<User> {
        if let Some(ref email) = update.email {
            if self.email_exists(email.as_str(), Some(id)).await {
                return Err(AppError::Conflict(format!(
                    "user {} already exists",
                    email.as_str()
                )));
            }
        }

        let mut store = self.store.write().await;
        let existing = store
            .get(&id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;

        let name = update
            .name
            .clone()
            .unwrap_or_else(|| existing.name().clone());
        let email = update
            .email
            .clone()
            .unwrap_or_else(|| existing.email().clone());
        let password_hash = update
            .password_hash
            .clone()
            .unwrap_or_else(|| existing.password_hash().clone());
        let role = update.role.clone().unwrap_or_else(|| existing.role());
        let updated_at = Utc::now();

        let updated = User::new(
            existing.id(),
            name,
            email,
            role,
            password_hash,
            existing.created_at(),
            updated_at,
        );

        store.insert(id, updated.clone());
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        let mut store = self.store.write().await;
        match store.remove(&id) {
            Some(_) => Ok(()),
            None => Err(AppError::NotFound(format!("user {id} not found"))),
        }
    }
}
