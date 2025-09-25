use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtManager {
    encoding: EncodingKey,
    decoding: DecodingKey,
    ttl: Duration,
}

impl JwtManager {
    pub fn new(secret: &str, ttl_minutes: i64) -> Self {
        let encoding = EncodingKey::from_secret(secret.as_bytes());
        let decoding = DecodingKey::from_secret(secret.as_bytes());
        let ttl = Duration::minutes(ttl_minutes);

        Self {
            encoding,
            decoding,
            ttl,
        }
    }

    pub fn generate(
        &self,
        user_id: Uuid,
        email: &str,
        role: &str,
    ) -> Result<TokenDetails, TokenError> {
        let now = Utc::now();
        let exp = now
            .checked_add_signed(self.ttl)
            .ok_or(TokenError::InvalidTtl)?
            .timestamp();

        let claims = Claims {
            sub: user_id,
            email: email.to_owned(),
            role: role.to_owned(),
            iat: now.timestamp(),
            exp,
        };

        let token = encode(&Header::default(), &claims, &self.encoding)?;
        let expires_at = DateTime::from_timestamp(exp, 0).ok_or(TokenError::InvalidTtl)?;

        Ok(TokenDetails { token, expires_at })
    }

    pub fn verify(&self, token: &str) -> Result<Claims, TokenError> {
        let validation = Validation::default();
        let token = decode::<Claims>(token, &self.decoding, &validation)?;

        Ok(token.claims)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct TokenDetails {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("invalid token: {0}")]
    Invalid(String),
    #[error("token expired")]
    Expired,
    #[error("invalid token ttl")]
    InvalidTtl,
}

impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        match error.kind() {
            ErrorKind::ExpiredSignature => Self::Expired,
            ErrorKind::InvalidToken | ErrorKind::InvalidSignature => {
                Self::Invalid(error.to_string())
            }
            _ => Self::Invalid(error.to_string()),
        }
    }
}
