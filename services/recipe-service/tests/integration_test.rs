use recipe_service::RecipeService;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

async fn setup_service(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) -> RecipeService {
    let pool = pool_options
        .connect_with(options.foreign_keys(true).create_if_missing(true))
        .await
        .unwrap();
    RecipeService::new(pool)
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("glass_of_water"))]
#[allow(clippy::float_cmp)]
async fn test_glass_of_water(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    let service = setup_service(pool_options, options).await;

    let all_titles = service.recipe_listing().await.unwrap();
    let id = all_titles
        .iter()
        .find_map(|recipe_listing| {
            if recipe_listing.title == "Glass of water" {
                Some(recipe_listing.id)
            } else {
                None
            }
        })
        .unwrap();

    let r = service.recipe(id).await.unwrap();

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

    let non_existing_id = 5;
    let r = service.recipe(non_existing_id).await;
    assert!(matches!(r, Err(recipe_service::RecipeError::NotFound)));
}

