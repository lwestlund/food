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

    let request = Request::get("/api/recipes").body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let v: Vec<models::RecipeListing> = serde_json::from_slice(&body).unwrap();
    assert_eq!(v.len(), 1);

    let recipe_listing = v.first().unwrap();
    assert_eq!(recipe_listing.id, 1);
    assert_eq!(recipe_listing.title, "Glass of water");
}

#[sqlx::test(fixtures("glass_of_water"))]
#[allow(clippy::float_cmp)]
async fn test_get_recipe_by_id(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    let pool = pool_options
        .connect_with(database::configure_connect_options(options))
        .await
        .unwrap();
    let app = server::app(pool);

    let request = Request::get("/api/recipes/1").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let r: models::Recipe = serde_json::from_slice(&body).unwrap();

    assert_eq!(r.title, "Glass of water");
    assert_eq!(r.description, "Refreshing, isn't it?");
    assert_eq!(r.meal_type, "Drink");
    assert_eq!(r.source_name, "Cool source");
    assert_eq!(r.source_url, None);
    assert_eq!(r.ingredients[0].quantity, 1.0);
    assert_eq!(r.ingredients[0].unit, "piece");
    assert_eq!(r.ingredients[0].name, "any drinking glass");
    assert_eq!(r.ingredients[1].quantity, 2.5);
    assert_eq!(r.ingredients[1].unit, "dl");
    assert_eq!(r.ingredients[1].name, "water");
    assert_eq!(r.instructions[0], "Pour the water into the glass.");
    assert_eq!(r.instructions[1], "Enjoy the nice water.");
    assert_eq!(
        r.creation_date,
        chrono::NaiveDate::from_ymd_opt(2025, 1, 19).unwrap()
    );
}

#[sqlx::test]
async fn test_404(pool: sqlx::SqlitePool) {
    let app = server::app(pool);

    let request = Request::get("/does-not-exist").body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
