use dioxus::fullstack::{FullstackContext, StatusCode};
use dioxus::prelude::*;

use crate::views::{Home, NavBar, NotFound, Recipe, RecipeList, Recipes};

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub(super) enum Route {
    #[layout(NavBar)]
    #[layout(ErrorLayout)]
        #[route("/", Home)]
        Home,
        #[nest("/recipes")]
            #[layout(Recipes)]
                #[route("/", RecipeList)]
                RecipeList,
                #[route("/:recipe", Recipe)]
                Recipe { recipe: String },
            #[end_layout]
        #[end_nest]
        #[route("/:..route", NotFound)]
        NotFound { route: Vec<String> },
}

#[component]
fn ErrorLayout() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |err: ErrorContext| {
                let http_error = FullstackContext::commit_error_status(err.error().unwrap());
                match http_error.status {
                    StatusCode::BAD_REQUEST => if let Some(message) = http_error.message {
                        rsx! { div { "{message}" } }
                    } else {
                        rsx! { div { "400 Bad Request" } }
                    }
                    StatusCode::UNAUTHORIZED => rsx! { div { "401 Unauthorized" } },
                    StatusCode::NOT_FOUND => rsx! { NotFound { route: Vec::new() } },
                    StatusCode::INTERNAL_SERVER_ERROR => rsx! {
                        h1 { "500 internal error" }
                        p { "We ran into a problem :(" }
                    },
                    _ => rsx! { div { "An unknown error occured" } }
                }
            },
            Outlet::<Route> {}
        }
    }
}
