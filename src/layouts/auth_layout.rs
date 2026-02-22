use dioxus::prelude::*;

use food::{User, backend};

use crate::router::Route;

#[derive(Clone)]
pub(crate) struct UserContext {
    pub user: Signal<Option<User>>,
}

#[component]
pub(crate) fn AuthLayout() -> Element {
    let user = use_server_future(backend::user::current_user)?;
    let user = user.unwrap().unwrap();
    let user = use_signal(|| user);
    use_context_provider(|| UserContext { user });

    rsx! {
        Outlet::<Route> {}
    }
}
