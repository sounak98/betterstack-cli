use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, parse_one};
use crate::types::SourceResource;
use crate::types::source::SourceUpdate;

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

    pub async fn update_source(&self, id: &str, update: &SourceUpdate) -> Result<SourceResource> {
        let path = format!("/sources/{id}");
        let resp =
            with_retry(|| async { Ok(self.patch(&path).json(update).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_source(&self, id: &str) -> Result<()> {
        let path = format!("/sources/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Delete failed ({status}): {body}");
        }
    }
}
