use food_backend::{database, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = database::from_env().await?;
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
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
