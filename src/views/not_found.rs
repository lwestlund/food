use dioxus::prelude::*;

#[component]
pub(crate) fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "404 Not Found" }
        p { "Looks like we could not find what you were looking for :(" }
    }
}
