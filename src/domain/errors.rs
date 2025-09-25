use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("validation error: {0}")]
    Validation(String),
}

impl DomainError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Validation(msg) => msg,
        }
    }
}
