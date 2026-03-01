mod user;

use anyhow::anyhow;
use sqlx::SqlitePool;
use user_service::UserService;

use user::User;

type SessionPool = axum_session_sqlx::SessionSqlitePool;
type Pool = SqlitePool;

pub(crate) type Session = axum_session_auth::AuthSession<User, Id, SessionPool, Pool>;
pub type AuthLayer = axum_session_auth::AuthSessionLayer<User, Id, SessionPool, Pool>;

pub type Id = i64;

#[async_trait::async_trait]
impl axum_session_auth::Authentication<User, Id, Pool> for User {
    #[tracing::instrument(err, skip(pool))]
    async fn load_user(userid: Id, pool: Option<&Pool>) -> anyhow::Result<User> {
        let pool = pool.ok_or_else(|| anyhow!("cannot get user information without a database"))?;
        let user_service = UserService::new(pool.to_owned());
        let user = user_service.user_by_id(userid).await?;
        Ok(user.into())
    }

    /// Is logged in.
    fn is_authenticated(&self) -> bool {
        true
    }

    /// Has been active recently in time.
    fn is_active(&self) -> bool {
        true
    }

    fn is_anonymous(&self) -> bool {
        false
    }
}
