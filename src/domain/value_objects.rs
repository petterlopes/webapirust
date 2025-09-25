use std::fmt;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::domain::errors::DomainError;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").expect("invalid email regex")
});

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse<S: AsRef<str>>(value: S) -> Result<Self, DomainError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DomainError::validation("email is required"));
        }
        if trimmed.len() > 190 {
            return Err(DomainError::validation(
                "email must be at most 190 characters",
            ));
        }
        if !EMAIL_REGEX.is_match(trimmed) {
            return Err(DomainError::validation("email has an invalid format"));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for EmailAddress {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EmailAddress::parse(s)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserName(String);

impl UserName {
    pub fn parse<S: AsRef<str>>(value: S) -> Result<Self, DomainError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DomainError::validation("name is required"));
        }
        if trimmed.len() > 120 {
            return Err(DomainError::validation(
                "name must be at most 120 characters",
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new<S: AsRef<str>>(value: S) -> Result<Self, DomainError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DomainError::validation("password hash cannot be empty"));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlainPassword(String);

impl PlainPassword {
    pub fn parse<S: AsRef<str>>(value: S) -> Result<Self, DomainError> {
        let raw = value.as_ref();
        if raw.len() < 12 {
            return Err(DomainError::validation(
                "password must be at least 12 characters",
            ));
        }
        let has_upper = raw.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = raw.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = raw.chars().any(|c| c.is_ascii_digit());
        let has_symbol = raw
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && !c.is_whitespace());

        if !(has_upper && has_lower && has_digit && has_symbol) {
            return Err(DomainError::validation(
                "password must include uppercase, lowercase, digit and symbol",
            ));
        }

        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
