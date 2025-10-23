#[cfg(feature = "server")]
pub mod database;

use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::server::axum::Extension;

use crate::models;

#[get("/api/recipes", pool: Extension<sqlx::SqlitePool>)]
pub async fn recipe_listing() -> Result<Vec<models::RecipeListing>> {
    let recipe_listing = database::recipe_listing(&pool).await?;
    Ok(recipe_listing)
}

#[get("/api/recipe", pool: Extension<sqlx::SqlitePool>)]
pub async fn recipe_by_id(recipe_id: i64) -> Result<models::Recipe> {
    let recipe = database::recipe(&pool, recipe_id).await?;
    Ok(recipe)
}
