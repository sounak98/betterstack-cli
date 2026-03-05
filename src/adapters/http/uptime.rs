use anyhow::{Context, Result, bail};

use super::HttpClient;
use super::retry::with_retry;
use crate::types::{CreateMonitorRequest, MonitorFilters, MonitorResource, SingleResponse};

/// Parse a single-resource response, or return an API error.
async fn parse_one<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("API error ({}): {}", status, body);
    }
    let single: SingleResponse<T> = resp.json().await.context("Failed to parse response")?;
    Ok(single.data)
}

/// Check response status and return an error with body if not successful.
async fn check_status(resp: reqwest::Response) -> Result<()> {
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("API error ({}): {}", status, body);
    }
    Ok(())
}

impl HttpClient {
    pub async fn list_monitors(&self, filters: &MonitorFilters) -> Result<Vec<MonitorResource>> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(ref url) = filters.url {
            params.push(("url", url.as_str()));
        }
        if let Some(ref name) = filters.pronounceable_name {
            params.push(("pronounceable_name", name.as_str()));
        }

        self.paginate_all("/monitors", &params).await
    }

    pub async fn get_monitor(&self, id: &str) -> Result<MonitorResource> {
        let path = format!("/monitors/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_monitor(&self, req: &CreateMonitorRequest) -> Result<MonitorResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/monitors").json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn pause_monitor(&self, id: &str) -> Result<MonitorResource> {
        let path = format!("/monitors/{id}");
        let body = serde_json::json!({ "paused": true });
        let resp = with_retry(|| async { Ok(self.patch(&path).json(&body).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn resume_monitor(&self, id: &str) -> Result<MonitorResource> {
        let path = format!("/monitors/{id}");
        let body = serde_json::json!({ "paused": false });
        let resp = with_retry(|| async { Ok(self.patch(&path).json(&body).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_monitor(&self, id: &str) -> Result<()> {
        let path = format!("/monitors/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }
}
