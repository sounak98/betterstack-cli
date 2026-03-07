use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{CreateSeverityRequest, SeverityResource, UpdateSeverityRequest};

impl HttpClient {
    pub async fn list_severities(&self) -> Result<Vec<SeverityResource>> {
        self.paginate_all("/urgencies", &[]).await
    }

    pub async fn get_severity(&self, id: &str) -> Result<SeverityResource> {
        let path = format!("/urgencies/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_severity(&self, req: &CreateSeverityRequest) -> Result<SeverityResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/urgencies").json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_severity(
        &self,
        id: &str,
        req: &UpdateSeverityRequest,
    ) -> Result<SeverityResource> {
        let path = format!("/urgencies/{id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_severity(&self, id: &str) -> Result<()> {
        let path = format!("/urgencies/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }
}
