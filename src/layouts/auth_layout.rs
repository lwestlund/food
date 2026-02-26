use dioxus::prelude::*;

use food::{api, models::User};

use crate::router::Route;

#[derive(Clone)]
pub(crate) struct UserContext {
    pub user: Signal<Option<User>>,
}

#[component]
pub(crate) fn AuthLayout() -> Element {
    let user = use_server_future(api::user::current_user)?;
    let user = user.unwrap().unwrap();
    let user = use_signal(|| user);
    use_context_provider(|| UserContext { user });

    rsx! {
        Outlet::<Route> {}
    }
}
