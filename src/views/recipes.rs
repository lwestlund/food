use crate::router::Route;
use food::{backend, models};

use dioxus::prelude::*;

#[component]
pub(crate) fn Recipes() -> Element {
    rsx! {
        h1 { "Recipes" }
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn RecipeList() -> Element {
    let response = use_server_future(backend::recipe_listing)?;
    let response_read = response.read();
    // SAFETY: If the future was still pending, it would have early returned
    // with 'suspended' on the `?` above.
    let Ok(recipe_listings) = response_read.as_ref().unwrap() else {
        return HttpError::internal_server_error("failed to list recipes")?;
    };

    rsx! {
        ul { id: "recipe-listings",
             for listing in &recipe_listings {
                 li { key: "{listing.id}",
                      class: "recipe-listing",
                      Link {
                          to: Route::Recipe {
                              recipe: recipe_listing_to_slug(listing),
                          },
                          "{listing.title}"
                      }
                 }
             }
        }
    }
}

fn recipe_listing_to_slug(listing: &models::RecipeListing) -> String {
    use heck::ToKebabCase as _;
    format!("{}-{}", listing.id, listing.title.to_kebab_case())
}

#[component]
pub(crate) fn Recipe(recipe: String) -> Element {
    let (id, _title) = recipe.split_once('-').or_not_found("recipe not found")?;
    let id = id
        .parse::<i64>()
        .or_bad_request(format!("bad recipe id `{id}`"))?;

    let response = use_server_future(move || backend::recipe_by_id(id))?;
    let response_read = response.read();
    // SAFETY: If the future was still pending, it would have early returned
    // with 'suspended' on the `?` above.
    let Ok(r) = response_read.as_ref().unwrap() else {
        return HttpError::not_found("no such recipe")?;
    };

    let expected_slug = recipe_listing_to_slug(&models::RecipeListing {
        id,
        title: r.title.clone(),
    });
    if expected_slug != recipe {
        return HttpError::not_found("no such recipe")?;
    }

    rsx! {
        div { id: "wrapper",
            div { id: "recipe",
                h2 { "{r.title}" }
                div { id: "meal-type",
                    p { "{r.meal_type.to_uppercase()}" }
                }
                p { "{r.description}" }

                div {
                    h4 { "Ingredients" }
                    ul { id: "ingredients",
                        for ingredient in &r.ingredients {
                            li { "{ingredient.quantity} {ingredient.unit} {ingredient.name}" }
                        }
                    }
                }

                div {
                    h4 { "Instructions" }
                    ol {
                        for instruction in &r.instructions {
                            li { "{instruction}" }
                        }
                    }
                }
            }

            div { id: "footer",
                if let Some(source_url) = r.source_url.as_ref() {
                    a { href: "{source_url}", "{r.source_name}" }
                } else {
                    "{r.source_name}"
                }
                ", {r.creation_date}"
            }
        }
    }
}
