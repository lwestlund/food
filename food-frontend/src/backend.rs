use std::net::IpAddr;

use axum::body::Bytes;
use reqwest::header;

use crate::mime;

#[derive(Clone)]
pub(crate) struct Backend {
    addr: IpAddr,
    port: u16,
    client: reqwest::Client,
}

impl Backend {
    pub(crate) fn new(addr: IpAddr, port: u16) -> Self {
        let client = reqwest::Client::new();
        Self { addr, port, client }
    }

    #[inline]
    fn format_url(&self, path: &str) -> String {
        const SCHEME: &str = "http";
        format!(
            "{SCHEME}://{addr}:{port}{path}",
            addr = self.addr,
            port = self.port
        )
    }

    async fn get_json<T>(&self, endpoint: &str) -> Result<T, BackendError>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        let bytes = self.get_json_bytes(endpoint).await?;
        let t: T = serde_json::from_slice(&bytes)?;
        Ok(t)
    }
    async fn get_json_bytes(&self, endpoint: &str) -> Result<Bytes, BackendError> {
        let url = self.format_url(endpoint);

        let response = self
            .client
            .get(url)
            .header(header::ACCEPT, mime::APPLICATION_JSON)
            .send()
            .await?
            .error_for_status()?;
        let Some(content_type) = response.headers().get(header::CONTENT_TYPE) else {
            return Err(BackendError::UnknownContentType(None));
        };
        if content_type != mime::APPLICATION_JSON {
            let content_type = String::from_utf8_lossy(content_type.as_bytes()).into_owned();
            return Err(BackendError::UnknownContentType(Some(content_type)));
        }
        let bytes = response.bytes().await?;
        Ok(bytes)
    }

    pub(crate) async fn get_recipe_listing(
        &self,
    ) -> Result<Vec<models::RecipeListing>, BackendError> {
        self.get_json("/api/recipes").await
    }

    pub(crate) async fn get_recipe_by_id(&self, id: i64) -> Result<models::Recipe, BackendError> {
        self.get_json(&format!("/api/recipes/{id}")).await
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum BackendError {
    #[error("error in request to backend: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("unknown content type in response from backend: {0:?}")]
    UnknownContentType(Option<String>),
    #[error("error deserializing response as json: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<Option<String>> for BackendError {
    fn from(value: Option<String>) -> Self {
        Self::UnknownContentType(value)
    }
}
