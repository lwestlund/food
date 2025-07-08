mod error;

use error::Result;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sqlx::SqlitePool;

use crate::{database, models};

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/recipes", get(get_recipes))
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

async fn get_recipes(State(pool): State<SqlitePool>) -> Result<Json<Vec<models::RecipeListing>>> {
    let recipes = database::all_recipe_titles(&pool).await?;
    Ok(Json(recipes))
}

async fn handler_404() -> StatusCode {
    StatusCode::NOT_FOUND
}
