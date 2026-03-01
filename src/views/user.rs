use dioxus::fullstack::Form;
use dioxus::prelude::*;
use food::api;

use crate::layouts::UserContext;
use crate::router::Route;

#[component]
pub(crate) fn User() -> Element {
    let nav = use_navigator();

    let user_ctx = use_context::<UserContext>();
    let user = user_ctx.user;

    let mut logout = use_action(api::user::logout);

    rsx! {
        div { class: "content",
            if let Some(user) = user() {
                h2 { "Hello, {user.username}" }
                p { "We're working on the user page here, come back soon! 😊" }
                button {
                    onclick: move |_| async move {
                        logout.call().await;
                        consume_context::<UserContext>().user.set(None);
                        nav.push(Route::Home);
                    },
                    "Log out"
                }
            } else {
                Login {}
            }
        }
    }
}

#[component]
fn Login() -> Element {
    let mut fail_state = use_signal(|| rsx! {});

    rsx! {
        div {
            form {
                onsubmit: move |evt: FormEvent| async move {
                    evt.prevent_default();
                    let login_form: api::user::LoginForm = evt.parsed_values().unwrap();
                    match api::user::login(Form(login_form)).await {
                        Ok(user) => {
                            consume_context::<UserContext>().user.set(Some(user));
                        }
                        Err(login_error) => {
                            use api::user::LoginError;
                            fail_state.set(rsx! {
                                a { class: "login-failed",
                                    match login_error {
                                        LoginError::InvalidCredentials => "Login failed",
                                        LoginError::Internal | LoginError::ServerFnError(_) => "Unexpected error",
                                    }
                                }
                            });
                        }
                    }
                },
                h1 { "Log in" }
                input {
                    name: "email",
                    placeholder: "E-mail",
                    r#type: "email",
                    required: true,
                    autofocus: true,
                    title: "",
                }
                input {
                    name: "password",
                    placeholder: "Password",
                    r#type: "password",
                }

                div { {fail_state} }
                div { class: "flex-row",
                    label { class: "checkbox-container",
                        input {
                            id: "stay-signed-in",
                            name: "stay_signed_in",
                            r#type: "checkbox",
                            value: "true",
                        }
                        span { class: "checkmark" }
                    }
                    label {
                        id: "stay-signed-in-label",
                        style: "margin-left: 5px;",
                        r#for: "stay-signed-in",
                        "Stay signed in"
                    }
                }
                button { width: "100%", r#type: "submit", "Log in" }
            }
        }
    }
}
