use anyhow::Context as _;
use food_backend::{database, server};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging()?;

    let pool = database::from_env().await?;
    let port = std::env::var("PORT").map_or_else(|_| Ok(3001), |port| port.parse())?;
    match server::serve(port, pool.clone()).await {
        Ok(()) => (),
        Err(err) => {
            pool.close().await;
            return Err(err);
        }
    }

    pool.close().await;
    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .context("Invalid logging directives from environment")?;
    let registry = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .compact();
    registry.init();
    Ok(())
}
