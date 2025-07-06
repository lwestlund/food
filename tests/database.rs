use food::database;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[sqlx::test(fixtures("glass_of_water"))]
#[allow(clippy::float_cmp)]
async fn glass_of_water(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    let pool = pool_options
        .connect_with(food::database::configure_connect_options(options))
        .await
        .unwrap();

    let all_titles = database::all_recipe_titles(&pool).await.unwrap();
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

    let r = database::recipe(&pool, id).await.unwrap();

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
