use sqlx::SqlitePool;
use user_service::{User, UserError, UserService};

#[must_use]
#[derive(Clone)]
pub struct AuthService {
    user_service: UserService,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            user_service: UserService::new(pool),
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<User, LoginError> {
        let user = self.user_service.user_by_email(email).await?;

        password_auth::verify_password(password, &user.password_hash)?;

        Ok(user)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("internal error")]
    Internal,
}

impl From<UserError> for LoginError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound => Self::InvalidCredentials,
            _ => Self::Internal,
        }
    }
}

impl From<password_auth::VerifyError> for LoginError {
    fn from(err: password_auth::VerifyError) -> Self {
        match err {
            password_auth::VerifyError::PasswordInvalid => Self::InvalidCredentials,
            password_auth::VerifyError::Parse(_) => Self::Internal,
        }
    }
}
