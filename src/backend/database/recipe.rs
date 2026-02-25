use crate::models;

pub(super) async fn ingredients(
    database: &sqlx::SqlitePool,
    recipe_id: i64,
) -> sqlx::Result<Vec<models::Ingredient>> {
    let ingredients: Vec<_> = sqlx::query!(
        r#"
        SELECT
            ri.quantity,
            m.unit,
            i.name
        FROM
            recipe_ingredient AS ri
        JOIN
            measurement AS m
            ON ri.measurement_id = m.id
        JOIN
            ingredient AS i
            ON ri.ingredient_id = i.id
        WHERE
            ri.recipe_id = ?
        ORDER BY
            ri.id;
        "#,
        recipe_id
    )
    .fetch_all(database)
    .await?
    .into_iter()
    .map(|record| models::Ingredient {
        quantity: record.quantity,
        unit: record.unit,
        name: record.name,
    })
    .collect();
    Ok(ingredients)
}

pub(super) async fn instructions(
    database: &sqlx::SqlitePool,
    recipe_id: i64,
) -> sqlx::Result<Vec<String>> {
    let instructions = sqlx::query!(
        r#"
        SELECT i.description
        FROM
            instruction AS i
        WHERE
            i.recipe_id = ?
        ORDER BY
            i.step_number;
        "#,
        recipe_id
    )
    .fetch_all(database)
    .await?
    .into_iter()
    .map(|record| record.description)
    .collect();
    Ok(instructions)
}
