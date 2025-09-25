use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::user::User;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserDto {
    #[schema(example = "Ada Lovelace")]
    pub name: String,
    #[schema(example = "ada@example.com")]
    pub email: String,
    #[schema(example = "Sup3rSecure!")]
    pub password: String,
    #[schema(example = "admin")]
    pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserDto {
    #[schema(example = "Grace Hopper")]
    pub name: Option<String>,
    #[schema(example = "grace@example.com")]
    pub email: Option<String>,
    #[schema(example = "N3wPassw0rd!")]
    pub password: Option<String>,
    #[schema(example = "viewer")]
    pub role: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id(),
            name: user.name().as_str().to_string(),
            email: user.email().as_str().to_string(),
            role: user.role().as_str().to_string(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}

impl UserResponseDto {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}
