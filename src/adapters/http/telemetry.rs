use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, parse_one};
use crate::types::SourceResource;

impl HttpClient {
    pub fn telemetry(token: &str) -> Self {
        Self::new("https://telemetry.betterstack.com/api/v1", token)
    }

    pub async fn list_sources(&self) -> Result<Vec<SourceResource>> {
        self.paginate_all("/sources", &[]).await
    }

    pub async fn get_source(&self, id: &str) -> Result<SourceResource> {
        let path = format!("/sources/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }
}
