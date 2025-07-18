use axum::{http::StatusCode, response::IntoResponse};
use maud::html;

use crate::backend::BackendError;

#[derive(Debug)]
pub(crate) enum AppError {
    NotFound,
    Internal {
        generic: String,
        detailed: Option<String>,
    },
}

impl From<BackendError> for AppError {
    fn from(err: BackendError) -> Self {
        match err {
            BackendError::Reqwest(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Self::NotFound,
                _ => Self::Internal {
                    generic: "We ran into an unexpected_problem.".into(),
                    detailed: Some(err.to_string()),
                },
            },
            BackendError::UnknownContentType(err) => Self::Internal {
                generic: "We ran into an unexpected_problem.".into(),
                detailed: err,
            },
            BackendError::Json(err) => Self::Internal {
                generic: "We ran into an unexpected_problem.".into(),
                detailed: Some(err.to_string()),
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct ErrorResponse {
    layout: super::Layout,
    error: AppError,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_string) = match &self.error {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "Looks like we could not find what you were looking for.",
            ),
            AppError::Internal { generic, detailed } => {
                eprintln!("Internal error: {detailed:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, generic.as_str())
            }
        };
        let title = format!(
            "{} - {}",
            status_code.as_str(),
            status_code.canonical_reason().unwrap_or("?")
        );
        let markup = {
            let code = html! { h1 { (status_code.as_str()) } };
            html! {
                    (code)
                    p { (error_string) }
            }
        };
        self.layout
            .with_status(&title)
            .render(&markup)
            .into_response()
    }
}

pub(crate) trait ResultWithLayout<T> {
    fn with_layout(self, layout: &super::Layout) -> Result<T, ErrorResponse>;
}

impl<T> ResultWithLayout<T> for std::result::Result<T, AppError> {
    fn with_layout(self, layout: &crate::Layout) -> Result<T, ErrorResponse> {
        match self {
            Self::Ok(ok) => Ok(ok),
            Self::Err(err) => Err(ErrorResponse {
                layout: layout.clone(),
                error: err,
            }),
        }
    }
}

pub(crate) trait ErrorWithLayout {
    fn with_layout(self, layout: &super::Layout) -> ErrorResponse;
}

impl ErrorWithLayout for AppError {
    fn with_layout(self, layout: &crate::Layout) -> ErrorResponse {
        ErrorResponse {
            layout: layout.clone(),
            error: self,
        }
    }
}
