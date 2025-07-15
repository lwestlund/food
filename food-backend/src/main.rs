use food_backend::{database, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
