use dioxus::prelude::*;

use crate::layouts::UserContext;
use crate::router::Route;

#[component]
pub(crate) fn NavbarLayout() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home, "Home" }
            Link { to: Route::RecipeList, "Recipes" }
            Profile {}
        }
        Outlet::<Route> {}
    }
}

#[component]
fn Profile() -> Element {
    let user_ctx = use_context::<UserContext>();
    match user_ctx.user.as_ref() {
        None => rsx! {
            Link { to: Route::UserPage, "Login" }
        },
        Some(user) => rsx! {
            Link { to: Route::UserPage, "{user.username}" }
        },
    }
}
