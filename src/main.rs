use food::database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = database::from_env().await?;
    match food::server::serve(pool.clone()).await {
        Ok(()) => (),
        Err(err) => {
            pool.close().await;
            return Err(err);
        }
    }

    pool.close().await;
    Ok(())
}
