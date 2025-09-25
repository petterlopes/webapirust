use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{
    Error as PasswordHashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("failed to hash password: {0}")]
    Hash(String),
    #[error("invalid credentials")]
    InvalidPassword,
}

impl From<PasswordHashError> for PasswordError {
    fn from(error: PasswordHashError) -> Self {
        Self::Hash(error.to_string())
    }
}

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(hash.to_string())
}

pub fn verify_password(expected_hash: &str, candidate: &str) -> Result<(), PasswordError> {
    let parsed_hash = PasswordHash::new(expected_hash)?;
    let argon2 = Argon2::default();

    match argon2.verify_password(candidate.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(()),
        Err(err) => match err {
            PasswordHashError::Password => Err(PasswordError::InvalidPassword),
            other => Err(PasswordError::from(other)),
        },
    }
}
