pub mod auth;
mod server_state;

pub use server_state::ServerState;

use sqlx::sqlite::SqliteConnectOptions;

pub fn configure_connect_options(connect_opts: SqliteConnectOptions) -> SqliteConnectOptions {
    connect_opts.foreign_keys(true).create_if_missing(true)
}
