use food::database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = database::from_env().await?;
    let all_recipe_titles = database::all_recipe_titles(&pool).await?;
    println!("{all_recipe_titles:?}");
    pool.close().await;
    Ok(())
}
