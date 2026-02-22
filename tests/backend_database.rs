use food::backend::{Database, database};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

async fn db_init(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) -> Database {
    let pool = pool_options
        .connect_with(database::configure_connect_options(options))
        .await
        .unwrap();
    Database::new(pool)
}

#[sqlx::test(fixtures("glass_of_water"))]
#[allow(clippy::float_cmp)]
async fn glass_of_water(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    let db = db_init(pool_options, options).await;

    let all_titles = db.recipe_listing().await.unwrap();
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

    let r = db.recipe(id).await.unwrap();

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
    let r = db.recipe(non_existing_id).await;
    assert!(matches!(r, Err(sqlx::Error::RowNotFound)));
}

#[sqlx::test]
async fn user(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    let db = db_init(pool_options, options).await;

    // Add user.
    let user_id = db
        .add_user("Mr Mittens", "mr@mittens.com", "foobarbaz")
        .await
        .unwrap();

    // Get user by ID.
    let user_by_id = db.user_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(user_by_id.id, user_id);
    assert_eq!(user_by_id.username, "Mr Mittens");
    assert_eq!(user_by_id.email, "mr@mittens.com");
    assert_eq!(user_by_id.password_hash, "foobarbaz");

    // Get user by email.
    let user_by_email = db.user_by_email("mr@mittens.com").await.unwrap();
    assert_eq!(user_by_email, Some(user_by_id));

    // Change password (hash).
    db.set_user_password_hash("mr@mittens.com", "CorrectHorseBatteryStaple")
        .await
        .unwrap();
    assert_eq!(
        db.user_by_id(user_id).await.unwrap().unwrap().password_hash,
        "CorrectHorseBatteryStaple"
    );

    // Delete user.
    db.delete_user("mr@mittens.com").await.unwrap();
    assert!(db.user_by_email("mr@mittens.com").await.unwrap().is_none());
    assert!(db.user_by_id(user_id).await.unwrap().is_none());
}
