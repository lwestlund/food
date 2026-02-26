use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus_fullstack::extract::State;

#[cfg(feature = "server")]
use crate::backend::Database;
use crate::models;

#[get("/api/recipes", db: State<Database>)]
pub async fn listing() -> Result<Vec<models::RecipeListing>> {
    let recipe_listing = db.recipe_listing().await?;
    Ok(recipe_listing)
}

#[get("/api/recipe", db: State<Database>)]
pub async fn by_id(recipe_id: i64) -> Result<models::Recipe> {
    let recipe = db.recipe(recipe_id).await?;
    Ok(recipe)
}
