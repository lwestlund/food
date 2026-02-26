use dioxus::fullstack::{FullstackContext, StatusCode};
use dioxus::prelude::*;

use crate::router::Route;
use crate::views;

#[component]
pub(crate) fn ErrorLayout() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |err: ErrorContext| {
                let http_error = FullstackContext::commit_error_status(err.error().unwrap());
                match http_error.status {
                    StatusCode::BAD_REQUEST => {
                        if let Some(message) = http_error.message {
                            rsx! {
                                div { "{message}" }
                            }
                        } else {
                            rsx! {
                                div { "400 Bad Request" }
                            }
                        }
                    }
                    StatusCode::UNAUTHORIZED => rsx! {
                        div { "401 Unauthorized" }
                    },
                    StatusCode::NOT_FOUND => rsx! {
                        views::NotFound { route: Vec::new() }
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => rsx! {
                        h1 { "500 internal error" }
                        p { "We ran into a problem :(" }
                    },
                    _ => rsx! {
                        div { "An unknown error occured" }
                    },
                }
            },
            Outlet::<Route> {}
        }
    }
}
