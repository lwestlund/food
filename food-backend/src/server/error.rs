use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = std::result::Result<T, ServerError>;

pub enum ServerError {
    NotFound,
    InternalError(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
            Self::InternalError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s).into_response(),
        }
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            err => Self::InternalError(err.to_string()),
        }
    }
}
