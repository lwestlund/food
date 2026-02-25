mod recipe;

use dioxus::fullstack::{FullstackContext, extract::FromRef};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use super::{User, auth};
use crate::models;

pub fn configure_connect_options(connect_opts: SqliteConnectOptions) -> SqliteConnectOptions {
    connect_opts.foreign_keys(true)
}

#[derive(Clone, Debug)]
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
            .map(|r| models::RecipeListing {
                id: r.id,
                title: r.title,
            })
            .collect();
        Ok(recipe_listings)
    }

    #[tracing::instrument(skip(self))]
    pub async fn user_by_id(&self, user_id: auth::Id) -> sqlx::Result<Option<User>> {
        let user = match sqlx::query!(
            r#"
            SELECT * FROM user
            WHERE id = ?;
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err),
        };

        Ok(Some(User {
            id: user.id,
            username: user.username,
            email: user.email,
            password_hash: user.password_hash,
        }))
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn user_by_email(&self, email: &str) -> sqlx::Result<Option<User>> {
        let user = match sqlx::query!(
            r#"
            SELECT * FROM user
            WHERE email = ?;
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err),
        };

        Ok(Some(User {
            id: user.id,
            username: user.username,
            email: user.email,
            password_hash: user.password_hash,
        }))
    }

    #[tracing::instrument(err)]
    pub async fn add_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> sqlx::Result<i64> {
        let result = sqlx::query!(
            r#"
            INSERT INTO user (username, email, password_hash)
            VALUES (?, ?, ?);
            "#,
            username,
            email,
            password_hash,
        )
        .execute(&self.pool)
        .await?;
        let id = result.last_insert_rowid();
        Ok(id)
    }

    pub async fn delete_user(&self, email: &str) -> sqlx::Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user
            WHERE email = ?;
            "#,
            email,
        )
        .execute(&self.pool)
        .await?;
        if result.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    pub async fn set_user_password_hash(
        &self,
        email: &str,
        password_hash: &str,
    ) -> sqlx::Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE user SET password_hash = ?
            WHERE email = ?;
            "#,
            password_hash,
            email
        )
        .execute(&self.pool)
        .await?;
        if result.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
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
