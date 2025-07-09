mod error;

use error::Result;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use sqlx::SqlitePool;

use crate::database;

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/recipes", get(get_recipes_list))
        .route("/api/recipes/{recipe_id}", get(get_recipe_by_id))
        .with_state(pool)
}

#[allow(clippy::missing_errors_doc)]
pub async fn serve(port: String, pool: SqlitePool) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("[::]:{port}")).await?;

    let app = app(pool);
    let app = app.fallback(handler_404);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_recipes_list(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<models::RecipeListing>>> {
    let recipes = database::all_recipe_titles(&pool).await?;
    Ok(Json(recipes))
}

async fn get_recipe_by_id(
    State(pool): State<SqlitePool>,
    Path(recipe_id): Path<i64>,
) -> Result<Json<models::Recipe>> {
    let recipe = database::recipe(&pool, recipe_id).await?;
    Ok(Json(recipe))
}

async fn handler_404() -> StatusCode {
    StatusCode::NOT_FOUND
}
