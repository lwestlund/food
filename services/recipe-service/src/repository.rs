use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::{Ingredient, Recipe, RecipeListing};

#[derive(Clone)]
pub(crate) struct RecipeRepository {
    pool: SqlitePool,
}

#[derive(thiserror::Error, Debug)]
pub enum RecipeError {
    #[error("recipe not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal integrity was compromised")]
    ConsistencyError,
}

impl RecipeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn recipe(&self, recipe_id: i64) -> Result<Recipe, RecipeError> {
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
                source AS s
                ON r.source_id = s.id
            JOIN
                meal_type AS mt
                ON r.meal_type_id = mt.id
            WHERE
                r.id = ?;
            "#,
            recipe_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RecipeError::NotFound)?;

        let ingredients = self.ingredients(recipe_id).await?;
        let instructions = self.instructions(recipe_id).await?;

        let recipe = Recipe {
            title: r.title,
            description: r.description,
            meal_type: r.meal_type,
            source_name: r.source_name,
            source_url: r.source_url,
            ingredients,
            instructions,
            creation_date: NaiveDate::parse_from_str(&r.creation_date, "%Y-%m-%d")
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
        };
        Ok(recipe)
    }

    pub async fn recipe_listing(&self) -> Result<Vec<RecipeListing>, RecipeError> {
        let r = sqlx::query!(
            r#"
            SELECT
                id,
                title
            FROM recipe;
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        let recipe_listings = r
            .into_iter()
            .map(|r| RecipeListing {
                id: r.id,
                title: r.title,
            })
            .collect();
        Ok(recipe_listings)
    }

    async fn ingredients(&self, recipe_id: i64) -> Result<Vec<Ingredient>, RecipeError> {
        let ingredients: Vec<_> = sqlx::query!(
            r#"
            SELECT
                ri.quantity,
                m.unit,
                i.name
            FROM
                recipe_ingredient AS ri
            JOIN
                measurement AS m
                ON ri.measurement_id = m.id
            JOIN
                ingredient AS i
                ON ri.ingredient_id = i.id
            WHERE
                ri.recipe_id = ?
            ORDER BY
                ri.id;
            "#,
            recipe_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|record| Ingredient {
            quantity: record.quantity,
            unit: record.unit,
            name: record.name,
        })
        .collect();
        Ok(ingredients)
    }

    async fn instructions(&self, recipe_id: i64) -> Result<Vec<String>, RecipeError> {
        let instructions = sqlx::query!(
            r#"
            SELECT i.description
            FROM
                instruction AS i
            WHERE
                i.recipe_id = ?
            ORDER BY
                i.step_number;
            "#,
            recipe_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|record| record.description)
        .collect();
        Ok(instructions)
    }
}
