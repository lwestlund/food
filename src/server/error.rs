use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = std::result::Result<T, ServerError>;

pub enum ServerError {
    InternalError(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InternalError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s).into_response(),
        }
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(err: sqlx::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}
