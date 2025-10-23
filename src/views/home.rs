use dioxus::prelude::*;

#[component]
pub(crate) fn Home() -> Element {
    rsx! {
        div { id: "title",
            h1 { "Food? Food!" }
        }
    }
}
