use crate::application::services::auth_service::AuthenticatedUser;
use crate::domain::entities::user::UserRole;
use crate::shared::error::AppError;

pub fn ensure_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    match user.role {
        UserRole::Admin => Ok(()),
        _ => Err(AppError::Forbidden("admin role required".to_string())),
    }
}

pub fn ensure_any(user: &AuthenticatedUser, allowed_roles: &[UserRole]) -> Result<(), AppError> {
    if allowed_roles.iter().any(|role| role == &user.role) {
        Ok(())
    } else {
        Err(AppError::Forbidden("insufficient role".to_string()))
    }
}
