use anyhow::anyhow;

use crate::backend;

type SessionPool = axum_session_sqlx::SessionSqlitePool;
type Pool = backend::Database;

pub(crate) type Session = axum_session_auth::AuthSession<backend::User, Id, SessionPool, Pool>;
pub type AuthLayer = axum_session_auth::AuthSessionLayer<backend::User, Id, SessionPool, Pool>;

pub type Id = i64;

#[async_trait::async_trait]
impl axum_session_auth::Authentication<backend::User, Id, backend::Database> for backend::User {
    #[tracing::instrument(err, skip(pool))]
    async fn load_user(
        userid: Id,
        pool: Option<&backend::Database>,
    ) -> anyhow::Result<backend::User> {
        let db = pool.ok_or_else(|| anyhow!("cannot get user information without a database"))?;
        let Some(user) = db.user_by_id(userid).await? else {
            anyhow::bail!("no such user");
        };
        Ok(user)
    }

    /// Is logged in.
    fn is_authenticated(&self) -> bool {
        todo!("is_authenticated")
    }

    /// Has been active recently in time.
    fn is_active(&self) -> bool {
        todo!("is_active")
    }

    fn is_anonymous(&self) -> bool {
        false
    }
}
