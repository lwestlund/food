use dioxus::prelude::*;
use dioxus_fullstack::AsStatusCode;
#[cfg(feature = "server")]
use dioxus_fullstack::extract::State;

#[cfg(feature = "server")]
use crate::backend::ServerState;
use crate::models;

#[get("/api/recipes", server_state: State<ServerState>)]
pub async fn listing() -> Result<Vec<models::RecipeListing>, RecipeListingError> {
    let recipe_listings = server_state.recipe.recipe_listing().await?;
    Ok(recipe_listings.into_iter().map(Into::into).collect())
}

#[get("/api/recipe", server_state: State<ServerState>)]
pub async fn by_id(recipe_id: i64) -> Result<models::Recipe, RecipeByIdError> {
    let recipe = server_state.recipe.recipe(recipe_id).await?;
    Ok(recipe.into())
}

pub use error::*;
mod error {
    use super::*;

    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum RecipeListingError {
        #[error("internal error")]
        Internal,
        #[error("internal server error")]
        ServerFnError(#[from] ServerFnError),
    }

    impl AsStatusCode for RecipeListingError {
        fn as_status_code(&self) -> StatusCode {
            match self {
                Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                Self::ServerFnError(err) => err.as_status_code(),
            }
        }
    }

    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum RecipeByIdError {
        #[error("recipe not found")]
        NotFound,
        #[error("internal error")]
        Internal,
        #[error("internal server error")]
        ServerFnError(#[from] ServerFnError),
    }

    impl AsStatusCode for RecipeByIdError {
        fn as_status_code(&self) -> StatusCode {
            match self {
                Self::NotFound => StatusCode::NOT_FOUND,
                Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                Self::ServerFnError(err) => err.as_status_code(),
            }
        }
    }
}

#[cfg(feature = "server")]
mod server {
    use super::*;

    impl From<recipe_service::RecipeListing> for models::RecipeListing {
        fn from(recipe_listing: recipe_service::RecipeListing) -> Self {
            Self {
                id: recipe_listing.id,
                title: recipe_listing.title,
            }
        }
    }

    impl From<recipe_service::Ingredient> for models::Ingredient {
        fn from(ingredient: recipe_service::Ingredient) -> Self {
            Self {
                quantity: ingredient.quantity,
                unit: ingredient.unit,
                name: ingredient.name,
            }
        }
    }

    impl From<recipe_service::Recipe> for models::Recipe {
        fn from(recipe: recipe_service::Recipe) -> Self {
            Self {
                title: recipe.title,
                description: recipe.description,
                meal_type: recipe.meal_type,
                source_name: recipe.source_name,
                source_url: recipe.source_url,
                ingredients: recipe.ingredients.into_iter().map(Into::into).collect(),
                instructions: recipe.instructions,
                creation_date: recipe.creation_date,
            }
        }
    }

    impl From<recipe_service::RecipeError> for RecipeListingError {
        fn from(err: recipe_service::RecipeError) -> Self {
            match err {
                recipe_service::RecipeError::NotFound => Self::Internal, // Should not happen for listing
                recipe_service::RecipeError::Database(_) => Self::Internal,
                recipe_service::RecipeError::ConsistencyError => Self::Internal,
            }
        }
    }

    impl From<recipe_service::RecipeError> for RecipeByIdError {
        fn from(err: recipe_service::RecipeError) -> Self {
            match err {
                recipe_service::RecipeError::NotFound => Self::NotFound,
                recipe_service::RecipeError::Database(_) => Self::Internal,
                recipe_service::RecipeError::ConsistencyError => Self::Internal,
            }
        }
    }
}
