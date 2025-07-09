mod backend;
mod mime;

use backend::Backend;

use std::net::{IpAddr, Ipv4Addr};

use anyhow::Context;
use axum::{
    Router,
    extract::{FromRequestParts, State},
    response::{IntoResponse, Response},
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
        .with_state(backend)
        .layer(LiveReloadLayer::new());
    let address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(address.clone()).await?;

    println!("Serving at http://{address}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root(State(backend): State<Backend>, layout: Layout) -> Response {
    let recipe_listing = backend.get_recipe_listing().await.unwrap();
    let listing = recipe_listing.into_iter().map(|listing| {
        html! {
            li {
                (listing.title)
            }
        }
    });
    let body = html! {
        h1 { "Food" }
        ul {
            @for listing in listing {
                (listing)
            }
        }
    };
    layout.render(&body)
}

struct Layout {
    title: String,
}

impl Layout {
    fn render(self, body: &Markup) -> Response {
        html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="utf-8";
                    title { (self.title) }
                }
                body {
                    (body)
                }
            }
        }
        .into_response()
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
    ) -> Result<Self, Self::Rejection> {
        let title = "Food".to_owned();
        Ok(Self { title })
    }
}
