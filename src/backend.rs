#[cfg(feature = "server")]
pub mod database;
#[cfg(feature = "server")]
pub use database::Database;

use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::server::axum::Extension;

use crate::models;

#[get("/api/recipes", db: Extension<Database>)]
pub async fn recipe_listing() -> Result<Vec<models::RecipeListing>> {
    let recipe_listing = db.recipe_listing().await?;
    Ok(recipe_listing)
}

#[get("/api/recipe", db: Extension<Database>)]
pub async fn recipe_by_id(recipe_id: i64) -> Result<models::Recipe> {
    let recipe = db.recipe(recipe_id).await?;
    Ok(recipe)
}
