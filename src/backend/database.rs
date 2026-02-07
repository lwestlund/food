mod recipe;

use dioxus::fullstack::{FullstackContext, extract::FromRef};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::models;

pub fn configure_connect_options(connect_opts: SqliteConnectOptions) -> SqliteConnectOptions {
    connect_opts.foreign_keys(true)
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn recipe(&self, recipe_id: i64) -> sqlx::Result<models::Recipe> {
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
        .fetch_one(&self.pool)
        .await?;
        let ingredients = recipe::ingredients(&self.pool, recipe_id).await?;
        let instructions = recipe::instructions(&self.pool, recipe_id).await?;
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

    pub async fn recipe_listing(&self) -> sqlx::Result<Vec<models::RecipeListing>> {
        let r = sqlx::query!("SELECT id, title FROM recipe")
            .fetch_all(&self.pool)
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
}

// Needed to be able to extract Database as an axum::extract::State in server functions.
// If a Database isn't added to the axum router, this will panic at runtime and cause a 500 for the
// client.
impl FromRef<FullstackContext> for Database {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<Database>().unwrap()
    }
}
