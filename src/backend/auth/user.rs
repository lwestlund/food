use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl From<user_service::User> for User {
    fn from(user: user_service::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            password_hash: user.password_hash,
        }
    }
}

#[allow(
    clippy::from_over_into,
    reason = "no one should be constructing a models user from a backend user, otherwise the type has leaked"
)]
impl Into<crate::models::User> for User {
    fn into(self) -> crate::models::User {
        crate::models::User {
            username: self.username,
            email: self.email,
        }
    }
}
