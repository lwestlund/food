mod login_form;

pub use login_form::LoginForm;

use dioxus::fullstack::Form;
#[cfg(feature = "server")]
use dioxus::fullstack::axum::extract::State;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::backend::{Database, auth};
use crate::models;

#[get("/api/user", auth: auth::Session, db: State<Database>)]
pub async fn current_user() -> Result<Option<models::User>> {
    let Some(current_user) = auth.current_user.as_ref() else {
        return Ok(None);
    };
    match db.user_by_id(current_user.id).await {
        Ok(Some(user)) => Ok(Some(user.into())),
        Ok(None) => Ok(None),
        Err(_err) => Ok(None),
    }
}

#[post("/api/user/login", auth: auth::Session, db: State<Database>)]
#[tracing::instrument(err, skip_all, fields(email = form.email))]
pub async fn login(form: Form<LoginForm>) -> Result<Result<models::User, LoginError>> {
    if let Some(user) = auth.current_user {
        println!("has a current user: {user:?}");
        let Some(user) = db.user_by_id(user.id).await.unwrap() else {
            tracing::error!("failed to find current user in database");
            return HttpError::internal_server_error("internal error")?;
        };
        return Ok(Ok(user.into()));
    } else {
        println!("has no current user");
    }
    let user = match db.user_by_email(&form.email).await {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(Err(LoginError::InvalidCredentials)),
        Err(err) => {
            tracing::error!("error getting user by email: {err}");
            return Ok(Err(LoginError::InternalError));
        }
    };

    match password_auth::verify_password(&form.password, &user.password_hash) {
        Ok(()) => Ok(()),
        Err(password_auth::VerifyError::PasswordInvalid) => HttpError::unauthorized("bad login"),
        Err(password_auth::VerifyError::Parse(parse_error)) => {
            tracing::error!("error when verifying user password: {parse_error}");
            HttpError::internal_server_error("internal error")
        }
    }?;
    auth.login_user(user.id);
    auth.remember_user(form.stay_signed_in);
    tracing::info!("logged in user {}", user.id);
    Ok(Ok(user.into()))
}

#[post("/api/user/logout", auth: auth::Session)]
pub async fn logout() -> Result<()> {
    auth.logout_user();
    Ok(())
}

#[post("/api/user/add", State(db): State<Database>)]
#[tracing::instrument(err, skip(db))]
pub async fn add_user(
    username: String,
    email: String,
    password: String,
) -> Result<Result<(), AddUserError>> {
    tracing::info!("got request to add user \"{username}\"");

    if let Err(err) = server::validate_password(&password) {
        return Ok(Err(err.into()));
    }

    let password_hash = password_auth::generate_hash(password);
    let id = match db.add_user(&username, &email, &password_hash).await {
        Ok(id) => id,
        Err(err) => return Ok(Err(err.into())),
    };
    tracing::info!("user added with id {id}");
    Ok(Ok(()))
}

#[post("/api/user/delete", db: State<Database>)]
pub async fn delete_user(email: String) -> Result<()> {
    match db.delete_user(&email).await {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::error!("{err}");
            HttpError::internal_server_error(err.to_string())?
        }
    }
}

#[patch("/api/user/change-password", db: State<Database>)]
pub async fn change_password(
    email: String,
    current_password: String,
    new_password: String,
) -> Result<Result<(), ChangePasswordError>> {
    let user = match db.user_by_email(&email).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpError::unauthorized("no such user")?,
        Err(err) => {
            tracing::error!("error getting user by email: {err}");
            return HttpError::internal_server_error("internal error")?;
        }
    };

    match password_auth::verify_password(current_password, &user.password_hash) {
        Ok(()) => Ok(()),
        Err(password_auth::VerifyError::PasswordInvalid) => {
            HttpError::forbidden("invalid password")
        }
        Err(password_auth::VerifyError::Parse(parse_error)) => {
            tracing::error!("error when verifying user password: {parse_error}");
            HttpError::internal_server_error("error verifying password")
        }
    }?;

    if let Err(err) = server::validate_password(&new_password) {
        return Ok(Err(err.into()));
    }

    let password_hash = password_auth::generate_hash(new_password);
    match db.set_user_password_hash(&user.email, &password_hash).await {
        Ok(()) => (),
        Err(sqlx::Error::RowNotFound) => {
            tracing::error!("user not found when setting password hash");
            HttpError::internal_server_error("user not found when setting new password")?;
        }
        Err(err) => {
            tracing::error!("error setting user password hash: {err}");
            HttpError::internal_server_error("error saving new password")?;
        }
    };

    Ok(Ok(()))
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum LoginError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("internal error")]
    InternalError,
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum AddUserError {
    #[error("password requirements failed: {0:?}")]
    PasswordRequirement(Vec<PasswordRequirement>),
    #[error("internal error")]
    Internal,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ChangePasswordError(Vec<PasswordRequirement>);

impl std::fmt::Display for ChangePasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "password requirements failed: {failed_reqs:?}",
            failed_reqs = self.0
        )
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum PasswordRequirement {
    PasswordTooShort { min_length: usize },
    PasswordTooLong { max_length: usize },
    NoLowerCase,
    NoUppercase,
    NoDigit,
    NoSpecial,
}

#[cfg(feature = "server")]
mod server {
    use super::*;

    impl From<Vec<PasswordRequirement>> for AddUserError {
        fn from(requirements: Vec<PasswordRequirement>) -> Self {
            Self::PasswordRequirement(requirements)
        }
    }

    impl From<sqlx::Error> for AddUserError {
        fn from(_value: sqlx::Error) -> Self {
            Self::Internal
        }
    }

    impl From<Vec<PasswordRequirement>> for ChangePasswordError {
        fn from(requirements: Vec<PasswordRequirement>) -> Self {
            Self(requirements)
        }
    }

    pub fn validate_password(password: &str) -> Result<(), Vec<PasswordRequirement>> {
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
}
