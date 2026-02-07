mod recipe;
pub use recipe::{recipe, recipe_listing};

use sqlx::sqlite::SqliteConnectOptions;

pub fn configure_connect_options(connect_opts: SqliteConnectOptions) -> SqliteConnectOptions {
    connect_opts.foreign_keys(true)
}
