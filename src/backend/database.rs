use sqlx::sqlite::SqliteConnectOptions;

use crate::models;

pub fn configure_connect_options(connect_opts: SqliteConnectOptions) -> SqliteConnectOptions {
    connect_opts.foreign_keys(true)
}

pub async fn recipe_listing(pool: &sqlx::SqlitePool) -> sqlx::Result<Vec<models::RecipeListing>> {
    let r = sqlx::query!("SELECT id, title FROM recipe")
        .fetch_all(pool)
        .await?;
    let recipe_listings = r
        .into_iter()
        .map(|r| models::RecipeListing {
            id: r.id,
            title: r.title,
        })
        .collect();

    Ok(recipe_listings)
}

pub async fn recipe(pool: &sqlx::SqlitePool, recipe_id: i64) -> sqlx::Result<models::Recipe> {
    let r = sqlx::query!(
        r#"
SELECT
    r.title,
    r.description,
    s.name AS source_name,
    s.url AS source_url,
    mt.type_name AS meal_type,
    r.creation_date
FROM
    recipe AS r
JOIN
    source AS s ON r.source_id = s.id
JOIN
    meal_type AS mt ON r.meal_type_id = mt.id
WHERE
    r.id = ?
"#,
        recipe_id
    )
    .fetch_one(pool)
    .await?;

    let ingredients = ingredients(pool, recipe_id).await?;
    let instructions = instructions(pool, recipe_id).await?;

    let recipe = models::Recipe {
        title: r.title,
        description: r.description,
        meal_type: r.meal_type,
        source_name: r.source_name,
        source_url: r.source_url,
        ingredients,
        instructions,
        creation_date: chrono::NaiveDate::parse_from_str(&r.creation_date, "%Y-%m-%d")
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
    };
    Ok(recipe)
}

async fn ingredients(
    database: &sqlx::SqlitePool,
    recipe_id: i64,
) -> sqlx::Result<Vec<models::Ingredient>> {
    let ingredients: Vec<_> = sqlx::query!(
        r#"
SELECT
    ri.quantity,
    m.unit,
    i.name
FROM
    recipe_ingredient AS ri
JOIN
    measurement AS m ON ri.measurement_id = m.id
JOIN
    ingredient AS i ON ri.ingredient_id = i.id
WHERE
    ri.recipe_id = ?
ORDER BY
    ri.id
"#,
        recipe_id
    )
    .fetch_all(database)
    .await?
    .into_iter()
    .map(|record| models::Ingredient {
        quantity: record.quantity,
        unit: record.unit,
        name: record.name,
    })
    .collect();
    Ok(ingredients)
}

async fn instructions(database: &sqlx::SqlitePool, recipe_id: i64) -> sqlx::Result<Vec<String>> {
    let instructions = sqlx::query!(
        r#"
SELECT
    i.description
FROM
    instruction AS i
WHERE
    i.recipe_id = ?
ORDER BY
    i.step_number
"#,
        recipe_id
    )
    .fetch_all(database)
    .await?
    .into_iter()
    .map(|record| record.description)
    .collect();
    Ok(instructions)
}
