use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use food_backend::{database, models, server};
use http_body_util::BodyExt as _;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tower::ServiceExt as _;

#[sqlx::test(fixtures("glass_of_water"))]
async fn test_get_recipes(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    let pool = pool_options
        .connect_with(database::configure_connect_options(options))
        .await
        .unwrap();
    let app = server::app(pool);

    let request = Request::get("/recipes").body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let v: Vec<models::RecipeListing> = serde_json::from_slice(&body).unwrap();
    assert_eq!(v.len(), 1);

    let recipe_listing = v.first().unwrap();
    assert_eq!(recipe_listing.id, 1);
    assert_eq!(recipe_listing.title, "Glass of water");
}

#[sqlx::test]
async fn test_404(pool: sqlx::SqlitePool) {
    let app = server::app(pool);

    let request = Request::get("/does-not-exist").body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
