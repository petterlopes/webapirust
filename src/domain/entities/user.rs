use std::error::Error;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::value_objects::{EmailAddress, PasswordHash, PlainPassword, UserName};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Viewer,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Viewer => "viewer",
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct UserRoleParseError(pub String);

impl fmt::Display for UserRoleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for UserRoleParseError {}

impl FromStr for UserRole {
    type Err = UserRoleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Self::Admin),
            "viewer" => Ok(Self::Viewer),
            _ => Err(UserRoleParseError(format!("invalid role: {value}"))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    name: UserName,
    email: EmailAddress,
    role: UserRole,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        name: UserName,
        email: EmailAddress,
        role: UserRole,
        password_hash: PasswordHash,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            email,
            role,
            password_hash,
            created_at,
            updated_at,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: Uuid,
        name: &str,
        email: &str,
        role: UserRole,
        password_hash: &str,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            name: UserName::parse(name)?,
            email: EmailAddress::parse(email)?,
            role,
            password_hash: PasswordHash::new(password_hash)?,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn email(&self) -> &EmailAddress {
        &self.email
    }

    pub fn role(&self) -> UserRole {
        self.role.clone()
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Clone, Debug)]
pub struct NewUser {
    pub name: UserName,
    pub email: EmailAddress,
    pub password_hash: PasswordHash,
    pub role: UserRole,
}

impl NewUser {
    pub fn build(
        name: UserName,
        email: EmailAddress,
        password_hash: PasswordHash,
        role: UserRole,
    ) -> Self {
        Self {
            name,
            email,
            password_hash,
            role,
        }
    }

    pub fn try_from_input(
        name: &str,
        email: &str,
        password: &PlainPassword,
        role: UserRole,
        hashed_password: &str,
    ) -> Result<Self, DomainError> {
        let _ = password; // ensures password already validated
        Ok(Self {
            name: UserName::parse(name)?,
            email: EmailAddress::parse(email)?,
            password_hash: PasswordHash::new(hashed_password)?,
            role,
        })
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn email(&self) -> &EmailAddress {
        &self.email
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn role(&self) -> UserRole {
        self.role.clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct UpdateUser {
    pub name: Option<UserName>,
    pub email: Option<EmailAddress>,
    pub password_hash: Option<PasswordHash>,
    pub role: Option<UserRole>,
}

impl UpdateUser {
    pub fn apply_name(mut self, name: UserName) -> Self {
        self.name = Some(name);
        self
    }

    pub fn apply_email(mut self, email: EmailAddress) -> Self {
        self.email = Some(email);
        self
    }

    pub fn apply_password_hash(mut self, password_hash: PasswordHash) -> Self {
        self.password_hash = Some(password_hash);
        self
    }

    pub fn apply_role(mut self, role: UserRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn name_str(&self) -> Option<&str> {
        self.name.as_ref().map(|value| value.as_str())
    }

    pub fn email_str(&self) -> Option<&str> {
        self.email.as_ref().map(|value| value.as_str())
    }

    pub fn password_hash_str(&self) -> Option<&str> {
        self.password_hash.as_ref().map(|value| value.as_str())
    }

    pub fn role(&self) -> Option<UserRole> {
        self.role.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.email.is_none()
            && self.password_hash.is_none()
            && self.role.is_none()
    }
}
