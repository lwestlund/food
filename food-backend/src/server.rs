mod error;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use error::Result;

use axum::{
    Json, Router,
    extract::{OriginalUri, Path, State},
    http::StatusCode,
    routing::get,
};
use sqlx::SqlitePool;
use tracing::{Level, instrument};

use crate::database;

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/recipes", get(get_recipes_list))
        .route("/api/recipes/{recipe_id}", get(get_recipe_by_id))
        .with_state(pool)
}

#[allow(clippy::missing_errors_doc)]
pub async fn serve(port: u16, pool: SqlitePool) -> anyhow::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = app(pool);
    let app = app.fallback(handler_404);

    tracing::info!("Serving at http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument(skip(pool), ret)]
async fn get_recipes_list(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<models::RecipeListing>>> {
    let recipes = database::all_recipe_titles(&pool).await?;
    Ok(Json(recipes))
}

#[instrument(skip(pool), ret)]
async fn get_recipe_by_id(
    State(pool): State<SqlitePool>,
    Path(recipe_id): Path<i64>,
) -> Result<Json<models::Recipe>> {
    let recipe = database::recipe(&pool, recipe_id).await?;
    Ok(Json(recipe))
}

#[instrument(ret(level = Level::ERROR))]
async fn handler_404(OriginalUri(uri): OriginalUri) -> StatusCode {
    StatusCode::NOT_FOUND
}
