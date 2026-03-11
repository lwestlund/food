mod app;
mod components;
mod layouts;
mod router;
mod views;

use app::App;

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        use anyhow::Context as _;
        use axum_session::{SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::AuthConfig;
        use axum_session_sqlx::SessionSqlitePool;
        use dioxus::server::axum::Extension;
        use food::backend::auth::AuthLayer;
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
        use std::str::FromStr as _;

        let pool = {
            let database_url = std::env::var("DATABASE_URL")
                .context("No database provided: please define `DATABASE_URL` and run again")?;
            tracing::debug!("Attempting to load database from {database_url}");
            let connect_opts = SqliteConnectOptions::from_str(&database_url)?;
            let connect_opts = food::backend::configure_connect_options(connect_opts);
            let pool_options = SqlitePoolOptions::new();
            pool_options
                .connect_with(connect_opts)
                .await
                .context("failed to connect to database")?
        };
        sqlx::migrate!()
            .run(&pool)
            .await
            .context("failed to run database migrations")?;

        let auth_layer = {
            let auth_config = AuthConfig::default();
            AuthLayer::new(Some(pool.clone())).with_config(auth_config)
        };

        let session_layer = {
            let session_store = SessionStore::<SessionSqlitePool>::new(
                Some(pool.clone().into()),
                SessionConfig::default().with_table_name("sessions"),
            )
            .await?;
            SessionLayer::new(session_store)
        };

        let server_state = food::backend::ServerState::new(pool);

        let router = dioxus::server::router(App)
            // Important that the auth layer gets added before the session layer for
            // the middleware to apply in the right order.
            .layer(auth_layer)
            .layer(session_layer)
            .layer(Extension(server_state));

        Ok(router)
    })
}
