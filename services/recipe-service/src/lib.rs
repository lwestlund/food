mod repository;

use chrono::NaiveDate;
use sqlx::SqlitePool;

pub use crate::repository::RecipeError;
use crate::repository::RecipeRepository;

#[must_use]
#[derive(Clone)]
pub struct RecipeService {
    repo: RecipeRepository,
}

impl RecipeService {
    pub fn new(pool: SqlitePool) -> Self {
        let repo = RecipeRepository::new(pool);
        Self { repo }
    }

    pub async fn recipe_listing(&self) -> Result<Vec<RecipeListing>, RecipeError> {
        self.repo.recipe_listing().await
    }

    pub async fn recipe(&self, recipe_id: i64) -> Result<Recipe, RecipeError> {
        self.repo.recipe(recipe_id).await
    }
}

#[derive(Clone, Debug)]
pub struct RecipeListing {
    pub id: i64,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub title: String,
    pub description: String,
    pub meal_type: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<String>,
    pub creation_date: NaiveDate,
}

#[derive(Clone, Debug)]
pub struct Ingredient {
    pub quantity: f64,
    pub unit: String,
    pub name: String,
}
