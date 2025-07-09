use std::sync::{Arc, RwLock};

use anyhow::{Context as _, anyhow, bail};
use axum::body::Bytes;
use http::{Request, header, uri::Authority};
use http_body_util::{BodyExt as _, Empty};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

#[derive(Clone)]
pub(crate) struct Backend {
    inner: Arc<RwLock<Inner>>,
}

type Http1Send = hyper::client::conn::http1::SendRequest<Empty<Bytes>>;

struct Inner {
    uri: http::Uri,
    sender: Http1Send,
}

impl Backend {
    pub(crate) async fn new(uri: http::Uri) -> anyhow::Result<Self> {
        if !matches!(uri.scheme_str(), Some("http")) {
            bail!("Only backends using http are supported");
        }

        // TODO(lovew): Need to re-initialize this if we restart the backend
        // while the frontend is running, otherwise the frontend will happily
        // try to use a closed connection.
        let stream = TcpStream::connect(
            uri.authority()
                .ok_or_else(|| anyhow!("No authority in backend url"))?
                .as_str(),
        )
        .await?;
        let io = TokioIo::new(stream);
        let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::spawn(async move {
            // TODO(lovew): Need to re-initialize if this returns at all I think.
            if let Err(err) = conn.await {
                println!("Backend connection failed: {err:?}");
            }
        });
        let inner = Inner { uri, sender };
        let inner = Arc::new(RwLock::new(inner));
        Ok(Self { inner })
    }

    #[inline]
    fn format_url(&self, path: &str) -> String {
        let uri = &self.inner.read().unwrap().uri;
        let scheme = uri.scheme().unwrap().as_str();
        let authority = uri.authority().unwrap().as_str();
        format!("{scheme}://{authority}{path}")
    }

    fn authority(&self) -> Authority {
        self.inner.read().unwrap().uri.authority().unwrap().clone()
    }

    fn send_request(
        &self,
        request: Request<Empty<axum::body::Bytes>>,
    ) -> impl Future<Output = Result<http::Response<hyper::body::Incoming>, hyper::Error>> {
        let mut lock = self.inner.write().unwrap();
        lock.sender.send_request(request)
    }

    pub(crate) async fn get_recipe_listing(&self) -> anyhow::Result<Vec<models::RecipeListing>> {
        let url = self.format_url("/api/recipes");
        let authority = self.authority();
        let request = Request::get(url)
            .header(header::HOST, authority.as_str())
            .header(header::ACCEPT, "application/json")
            .body(Empty::<axum::body::Bytes>::new())
            .context("Failed to construct request")?;

        let response = self.send_request(request).await?;
        let body = response.into_body().collect().await?.to_bytes();

        let recipe_listing = serde_json::from_slice(&body)?;
        Ok(recipe_listing)
    }
}
