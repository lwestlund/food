use std::net::IpAddr;

use anyhow::bail;
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

    pub(crate) async fn get_recipe_listing(&self) -> anyhow::Result<Vec<models::RecipeListing>> {
        let url = self.format_url("/api/recipes");

        let response = self
            .client
            .get(url)
            .header(header::ACCEPT, mime::APPLICATION_JSON)
            .send()
            .await?;
        let Some(content_type) = response.headers().get(header::CONTENT_TYPE) else {
            bail!("response does not have a content type");
        };
        if content_type != mime::APPLICATION_JSON {
            bail!("Did not get json back");
        }
        let bytes = response.bytes().await?;

        let recipe_listing = serde_json::from_slice(&bytes)?;
        Ok(recipe_listing)
    }
}
