use dioxus::prelude::*;

#[component]
pub(crate) fn Home() -> Element {
    rsx! {
        div { id: "title", class: "content",
            h1 { "Food? Food!" }
        }
    }
}
