use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeListing {
    pub id: i64,
    pub title: String,
}

#[derive(Debug)]
pub struct Recipe {
    pub title: String,
    pub description: String,
    pub meal_type: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<String>,
    pub creation_date: chrono::NaiveDate,
}

#[derive(Debug)]
pub struct Ingredient {
    pub quantity: f64,
    pub unit: String,
    pub name: String,
}
