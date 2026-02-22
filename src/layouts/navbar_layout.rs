use dioxus::prelude::*;

use crate::router::Route;

#[component]
pub(crate) fn NavbarLayout() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home, "Home" }
            Link { to: Route::RecipeList, "Recipes" }
        }
        Outlet::<Route> {}
    }
}
