use axum::{
    Router,
    extract::FromRequestParts,
    response::{IntoResponse, Response},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());

    let app = Router::new()
        .route("/", get(hello_world))
        .layer(LiveReloadLayer::new());
    let address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(address.clone()).await?;

    println!("Serving at http://{address}/");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello_world(layout: Layout) -> Response {
    let body = html! {
        h1 { "Food" }
        p { "Hello, world!" }
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
