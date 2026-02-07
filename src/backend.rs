#[cfg(feature = "server")]
pub mod database;
#[cfg(feature = "server")]
pub use database::Database;

mod api;
pub use api::*;
