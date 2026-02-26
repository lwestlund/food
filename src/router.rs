use dioxus::prelude::*;

use crate::layouts::{AuthLayout, ErrorLayout, NavbarLayout};
use crate::views;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub(super) enum Route {
    #[layout(AuthLayout)]
    #[layout(NavbarLayout)]
    #[layout(ErrorLayout)]
        #[route("/", views::Home)]
        Home,
        #[nest("/recipes")]
            #[layout(views::Recipes)]
                #[route("/", views::RecipeList)]
                RecipeList,
                #[route("/:recipe", views::Recipe)]
                Recipe { recipe: String },
            #[end_layout]
        #[end_nest]
        #[route("/user", views::User)]
        UserPage,
        #[route("/:..route", views::NotFound)]
        NotFound { route: Vec<String> },
}
