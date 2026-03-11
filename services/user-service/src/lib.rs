mod repository;
mod user;

pub use repository::UserError;
pub use user::User;

use sqlx::SqlitePool;

use crate::repository::UserRepository;

#[must_use]
#[derive(Clone)]
pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new(pool: SqlitePool) -> Self {
        let repo = UserRepository::new(pool);
        Self { repo }
    }

    pub async fn user_by_email(&self, email: &str) -> Result<User, UserError> {
        self.repo.user_by_email(email).await
    }

    pub async fn user_by_id(&self, user_id: i64) -> Result<User, UserError> {
        self.repo.user_by_id(user_id).await
    }

    pub async fn add_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<i64, AddUserError> {
        validate_password(password)?;

        let password_hash = password_auth::generate_hash(password);
        let id = self.repo.add_user(username, email, &password_hash).await?;
        Ok(id)
    }

    pub async fn delete_user(&self, email: &str) -> Result<(), DeleteUserError> {
        self.repo.delete_user(email).await?;
        Ok(())
    }

    pub async fn change_password(
        &self,
        email: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), ChangePasswordError> {
        let user = self.repo.user_by_email(email).await?;

        password_auth::verify_password(current_password, &user.password_hash)?;

        validate_password(new_password)?;

        let password_hash = password_auth::generate_hash(new_password);
        self.repo
            .set_user_password_hash(&user.email, &password_hash)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddUserError {
    #[error("user already exists")]
    AlreadyExists,
    #[error("password requirements failed: {0:?}")]
    PasswordRequirement(Vec<PasswordRequirement>),
    #[error("internal error")]
    Internal,
}

impl From<UserError> for AddUserError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::AlreadyExists => Self::AlreadyExists,
            _ => Self::Internal,
        }
    }
}

impl From<Vec<PasswordRequirement>> for AddUserError {
    fn from(requirements: Vec<PasswordRequirement>) -> Self {
        Self::PasswordRequirement(requirements)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PasswordRequirement {
    PasswordTooShort { min_length: usize },
    PasswordTooLong { max_length: usize },
    NoLowerCase,
    NoUppercase,
    NoDigit,
    NoSpecial,
}

fn validate_password(password: &str) -> Result<(), Vec<PasswordRequirement>> {
    const MINIMUM_PASSWORD_LENGTH: usize = 14;
    const MAXIMUM_PASSWORD_LENGTH: usize = 128;

    let mut failed_requirements = Vec::new();

    let length = password.len();
    if length < MINIMUM_PASSWORD_LENGTH {
        failed_requirements.push(PasswordRequirement::PasswordTooShort {
            min_length: MINIMUM_PASSWORD_LENGTH,
        });
    }
    if length > MAXIMUM_PASSWORD_LENGTH {
        failed_requirements.push(PasswordRequirement::PasswordTooLong {
            max_length: MAXIMUM_PASSWORD_LENGTH,
        });
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        failed_requirements.push(PasswordRequirement::NoLowerCase);
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        failed_requirements.push(PasswordRequirement::NoUppercase);
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        failed_requirements.push(PasswordRequirement::NoDigit);
    }
    if !password.chars().any(|c| c.is_ascii_punctuation()) {
        failed_requirements.push(PasswordRequirement::NoSpecial);
    }

    if !failed_requirements.is_empty() {
        return Err(failed_requirements);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteUserError {
    #[error("user not found")]
    NotFound,
    #[error("internal error")]
    Internal,
}

impl From<UserError> for DeleteUserError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound => Self::NotFound,
            _ => Self::Internal,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChangePasswordError {
    #[error("password requirements failed: {0:?}")]
    PasswordRequirements(Vec<PasswordRequirement>),
    #[error("user not found")]
    UserNotFound,
    #[error("wrong current password")]
    WrongCurrentPassword,
    #[error("internal error")]
    Internal,
}

impl From<password_auth::VerifyError> for ChangePasswordError {
    fn from(err: password_auth::VerifyError) -> Self {
        match err {
            password_auth::VerifyError::PasswordInvalid => Self::WrongCurrentPassword,
            password_auth::VerifyError::Parse(_parse_error) => Self::Internal,
        }
    }
}

impl From<UserError> for ChangePasswordError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound => ChangePasswordError::UserNotFound,
            _ => ChangePasswordError::Internal,
        }
    }
}

impl From<Vec<PasswordRequirement>> for ChangePasswordError {
    fn from(requirements: Vec<PasswordRequirement>) -> Self {
        Self::PasswordRequirements(requirements)
    }
}
