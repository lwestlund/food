mod error;

use error::Result;

use axum::{Extension, Json, Router, http::StatusCode, routing::get};
use sqlx::SqlitePool;

use crate::{database, models};

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/recipes", get(get_recipes))
        .layer(Extension(pool))
}

#[allow(clippy::missing_errors_doc)]
pub async fn serve(pool: SqlitePool) -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("[::]:{port}")).await?;

    let app = app(pool);
    let app = app.fallback(handler_404);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_recipes(pool: Extension<SqlitePool>) -> Result<Json<Vec<models::RecipeListing>>> {
    let recipes = database::all_recipe_titles(&pool).await?;
    Ok(Json(recipes))
}

async fn handler_404() -> StatusCode {
    StatusCode::NOT_FOUND
}
