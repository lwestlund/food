mod backend;
mod error;
mod mime;

use backend::Backend;
use error::{AppError, ErrorResponse, ErrorWithLayout as _, ResultWithLayout as _};
use heck::ToKebabCase;
use tower_http::services::ServeFile;

use std::net::{IpAddr, Ipv4Addr};

use anyhow::Context;
use axum::{
    Router,
    extract::{FromRequestParts, Path, State},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};
use tower_livereload::LiveReloadLayer;

const DEFAULT_BACKEND_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
const DEFAULT_BACKEND_PORT: u16 = 3001;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let backend = {
        let backend_addr: IpAddr = std::env::var("BACKEND_HOST").map_or_else(
            |_| Ok(DEFAULT_BACKEND_HOST),
            |addr| addr.parse().context("Failed to parse BACKEND_HOST"),
        )?;
        let backend_port: u16 = std::env::var("BACKEND_PORT").map_or_else(
            |_| Ok(DEFAULT_BACKEND_PORT),
            |port| port.parse().context("Failed to parse BACKEND_PORT"),
        )?;
        Backend::new(backend_addr, backend_port)
    };

    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let app = Router::new()
        .route("/", get(root))
        .route("/recipes/{recipe}", get(recipe))
        // TODO(lovew): Bundle the CSS with the application, this depends on
        // that the file exists on disk even at runtime.
        .route_service(
            "/styles.css",
            ServeFile::new(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/styles.css")),
        )
        .fallback(handler_404)
        .with_state(backend)
        .layer(LiveReloadLayer::new());
    let address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(address.clone()).await?;

    println!("Serving at http://{address}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root(State(backend): State<Backend>, layout: Layout) -> Result<Markup, ErrorResponse> {
    let recipe_listing = backend
        .get_recipe_listing()
        .await
        .map_err(Into::into)
        .with_layout(&layout)?;
    let listing = recipe_listing.into_iter().map(|listing| {
        let slug = recipe_id_title_to_slug(listing.id, &listing.title);
        let path = format!("/recipes/{slug}");
        html! {
            li {
                a href=(path){
                    (listing.title)
                }
            }
        }
    });
    let body = html! {
        h1 { "Welcome to my recipes" }
        p { "We've got:" }
        ul {
            @for listing in listing {
                (listing)
            }
        }
    };
    Ok(layout.render(&body))
}

async fn recipe(
    State(backend): State<Backend>,
    layout: Layout,
    Path(recipe_slug): Path<String>,
) -> Result<Markup, ErrorResponse> {
    let (id, _title) = recipe_slug
        .split_once('-')
        .ok_or(AppError::NotFound.with_layout(&layout))?;
    let id: i64 = id
        .parse()
        .map_err(|_| AppError::NotFound.with_layout(&layout))?;

    let recipe = backend
        .get_recipe_by_id(id)
        .await
        .map_err(Into::into)
        .with_layout(&layout)?;

    let expected_slug = recipe_id_title_to_slug(id, &recipe.title);
    if expected_slug != recipe_slug {
        return Err(AppError::NotFound.with_layout(&layout));
    }

    let body = recipe.render();

    Ok(layout.render(&body))
}

fn recipe_id_title_to_slug(id: i64, title: &str) -> String {
    format!("{}-{}", id, title.to_kebab_case())
}

async fn handler_404(layout: Layout) -> ErrorResponse {
    AppError::NotFound.with_layout(&layout)
}

#[derive(Clone)]
struct Layout {
    title: String,
}

impl Layout {
    fn render(self, body: &Markup) -> Markup {
        html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="utf-8";
                    title { (self.title) }
                    link rel="stylesheet" href="/styles.css";
                }
                body {
                    nav {
                        a href="/" { "Food" }
                    }
                    (body)
                }
            }
        }
    }

    fn with_status(mut self, status: &str) -> Self {
        const SEPARATOR: &str = " | ";
        self.title.reserve(status.len() + SEPARATOR.len());
        self.title.insert_str(0, SEPARATOR);
        self.title.insert_str(0, status);
        self
    }
}

trait Render {
    fn render(self) -> Markup;
}

impl Render for models::Recipe {
    fn render(self) -> Markup {
        html! {
            div id="wrapper" {
                div id="recipe" {
                    h2 { (self.title) }
                    div id="meal-type" {
                        p  { (self.meal_type.to_uppercase()) }
                    }
                    p { (self.description) }

                    div {
                        h4 { "Ingredients" }
                        div {
                            @for ingredient in self.ingredients {
                                div {
                                    "\u{2023} " (ingredient.quantity) " " (ingredient.unit) " " (ingredient.name)
                                }
                            }
                        }
                    }

                    div {
                        h4 { "Instructions" }
                        div {
                            @for (i, instruction) in self.instructions.iter().enumerate() {
                                div { (i+1) ". " (instruction) }
                            }
                        }
                    }
                }

                div id="footer" {
                    @if let Some(source_url) = self.source_url {
                        a href=(source_url) { (self.source_name) }
                    } @else {
                        (self.source_name)
                    }
                    ", " (self.creation_date)
                }
            }
        }
    }
}

impl<S> FromRequestParts<S> for Layout
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let title = "Food".to_owned();
        Ok(Self { title })
    }
}
