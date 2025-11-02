mod app;
mod components;
mod router;
mod views;

use app::App;

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        use anyhow::Context as _;
        use dioxus::server::axum::Extension;
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
        use std::str::FromStr as _;

        let pool = {
            let database_url = std::env::var("DATABASE_URL")
                .context("No database provided: please define `DATABASE_URL` and run again")?;
            tracing::debug!("Attempting to load database from {database_url}");
            let connect_opts = SqliteConnectOptions::from_str(&database_url)?;
            let connect_opts = food::backend::database::configure_connect_options(connect_opts);
            let pool_options = SqlitePoolOptions::new();
            pool_options.connect_with(connect_opts).await?
        };

        Ok(dioxus::server::router(App).layer(Extension(pool)))
    })
}
