mod login_form;

pub use login_form::LoginForm;

#[cfg(feature = "server")]
use dioxus::fullstack::axum::extract::State;
use dioxus::fullstack::{AsStatusCode, Form};
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::backend::{ServerState, auth};
use crate::models;

#[get("/api/user", auth: auth::Session, server_state: State<ServerState>)]
pub async fn current_user() -> Result<Option<models::User>> {
    let Some(current_user) = auth.current_user.as_ref() else {
        return Ok(None);
    };
    match server_state.user.user_by_id(current_user.id).await {
        Ok(user) => Ok(Some(user.into())),
        Err(_err) => Ok(None),
    }
}

#[post("/api/user/login", auth: auth::Session, server_state: State<ServerState>)]
#[tracing::instrument(skip_all, fields(email = form.email), err)]
pub async fn login(form: Form<LoginForm>) -> Result<models::User, LoginError> {
    let user = server_state.auth.login(&form.email, &form.password).await?;
    auth.login_user(user.id);
    auth.remember_user(form.stay_signed_in);
    tracing::info!("logged in user {}", user.id);
    Ok(user.into())
}

#[post("/api/user/logout", auth: auth::Session)]
pub async fn logout() -> Result<()> {
    auth.logout_user();
    Ok(())
}

#[post("/api/user/add", server_state: State<ServerState>)]
#[tracing::instrument(skip_all, fields(email = email), err)]
pub async fn add_user(
    username: String,
    email: String,
    password: String,
) -> Result<(), AddUserError> {
    server_state
        .user
        .add_user(&username, &email, &password)
        .await?;
    Ok(())
}

#[post("/api/user/delete", server_state: State<ServerState>)]
#[tracing::instrument(skip(server_state), err)]
pub async fn delete_user(email: String) -> ServerFnResult<()> {
    server_state
        .user
        .delete_user(&email)
        .await
        .or_internal_server_error("internal error")?;
    Ok(())
}

#[patch("/api/user/change-password", server_state: State<ServerState>)]
#[tracing::instrument(skip_all, fields(email = email), err)]
pub async fn change_password(
    email: String,
    current_password: String,
    new_password: String,
) -> Result<(), ChangePasswordError> {
    server_state
        .user
        .change_password(&email, &current_password, &new_password)
        .await?;
    Ok(())
}

pub use error::*;
mod error {
    use super::*;

    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum LoginError {
        #[error("invalid credentials")]
        InvalidCredentials,
        #[error("internal error")]
        Internal,
        #[error("internal server error")]
        ServerFnError(#[from] ServerFnError),
    }

    impl AsStatusCode for LoginError {
        fn as_status_code(&self) -> StatusCode {
            match self {
                Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
                Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                Self::ServerFnError(err) => err.as_status_code(),
            }
        }
    }

    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum AddUserError {
        #[error("password requirements failed: {0:?}")]
        PasswordRequirement(Vec<PasswordRequirement>),
        #[error("internal error")]
        Internal,
        #[error("internal server error")]
        ServerFnError(#[from] ServerFnError),
    }

    impl AsStatusCode for AddUserError {
        fn as_status_code(&self) -> StatusCode {
            match self {
                Self::PasswordRequirement(_) => StatusCode::BAD_REQUEST,
                Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                Self::ServerFnError(err) => err.as_status_code(),
            }
        }
    }

    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum ChangePasswordError {
        #[error("password requirements failed: {0:?}")]
        PasswordRequirements(Vec<PasswordRequirement>),
        #[error("wrong current password")]
        WrongCurrentPassword,
        #[error("internal error")]
        Internal,
        #[error("internal server error")]
        ServerFnError(#[from] ServerFnError),
    }

    impl AsStatusCode for ChangePasswordError {
        fn as_status_code(&self) -> StatusCode {
            match self {
                Self::PasswordRequirements(_) => StatusCode::BAD_REQUEST,
                Self::WrongCurrentPassword => StatusCode::UNAUTHORIZED,
                Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                Self::ServerFnError(err) => err.as_status_code(),
            }
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
}

#[cfg(feature = "server")]
mod server {
    use super::*;

    // Allowed to contstruct models::User on the server, not elsewhere.
    impl From<user_service::User> for models::User {
        fn from(user: user_service::User) -> Self {
            Self {
                username: user.username,
                email: user.email,
            }
        }
    }

    impl From<auth_service::LoginError> for LoginError {
        fn from(err: auth_service::LoginError) -> Self {
            match err {
                auth_service::LoginError::InvalidCredentials => Self::InvalidCredentials,
                auth_service::LoginError::Internal => Self::Internal,
            }
        }
    }

    impl From<user_service::PasswordRequirement> for PasswordRequirement {
        fn from(requirement: user_service::PasswordRequirement) -> Self {
            use user_service::PasswordRequirement as ServiceRequirement;
            match requirement {
                ServiceRequirement::PasswordTooShort { min_length } => {
                    Self::PasswordTooShort { min_length }
                }
                ServiceRequirement::PasswordTooLong { max_length } => {
                    Self::PasswordTooLong { max_length }
                }
                ServiceRequirement::NoLowerCase => Self::NoLowerCase,
                ServiceRequirement::NoUppercase => Self::NoUppercase,
                ServiceRequirement::NoDigit => Self::NoDigit,
                ServiceRequirement::NoSpecial => Self::NoSpecial,
            }
        }
    }

    impl From<user_service::AddUserError> for AddUserError {
        fn from(err: user_service::AddUserError) -> Self {
            use user_service::AddUserError as ServiceError;
            match err {
                ServiceError::AlreadyExists => Self::Internal,
                ServiceError::Internal => Self::Internal,
                ServiceError::PasswordRequirement(reqs) => {
                    Self::PasswordRequirement(reqs.into_iter().map(Into::into).collect())
                }
            }
        }
    }

    impl From<user_service::ChangePasswordError> for ChangePasswordError {
        fn from(err: user_service::ChangePasswordError) -> Self {
            use user_service::ChangePasswordError as ServiceError;
            match err {
                ServiceError::PasswordRequirements(requirements) => {
                    Self::PasswordRequirements(requirements.into_iter().map(Into::into).collect())
                }
                ServiceError::WrongCurrentPassword => Self::WrongCurrentPassword,
                ServiceError::UserNotFound => Self::Internal,
                ServiceError::Internal => Self::Internal,
            }
        }
    }
}
