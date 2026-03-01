use sqlx::SqlitePool;

use crate::User;

#[derive(Clone)]
pub(crate) struct UserRepository {
    pool: SqlitePool,
}

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("user already exists")]
    AlreadyExists,
    #[error("user not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal integrity was compromised")]
    ConsistencyError,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<i64, UserError> {
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
        .await;
        match result {
            Ok(query_result) => Ok(query_result.last_insert_rowid()),
            Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                Err(UserError::AlreadyExists)
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn delete_user(&self, email: &str) -> Result<(), UserError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user
            WHERE email = ?;
            "#,
            email,
        )
        .execute(&self.pool)
        .await?;
        match result.rows_affected() {
            0 => Err(UserError::NotFound),
            1 => Ok(()),
            _ => Err(UserError::ConsistencyError),
        }
    }

    pub async fn user_by_email(&self, email: &str) -> Result<User, UserError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM user
            WHERE email = ?;
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    pub async fn user_by_id(&self, user_id: i64) -> Result<User, UserError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM user
            WHERE id = ?;
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    pub async fn set_user_password_hash(
        &self,
        email: &str,
        password_hash: &str,
    ) -> Result<(), UserError> {
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
        match result.rows_affected() {
            0 => Err(UserError::NotFound),
            1 => Ok(()),
            _ => Err(UserError::ConsistencyError),
        }
    }
}
