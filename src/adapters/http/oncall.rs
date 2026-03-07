use anyhow::{Context, Result, bail};

use super::retry::with_retry;
use super::{HttpClient, check_status, format_error_body};
use crate::types::{
    CreateOnCallEventRequest, CreateOnCallRequest, OnCallDetailResponse, OnCallEventResource,
    OnCallEventsResponse, OnCallResource,
};

impl HttpClient {
    pub async fn list_oncalls(&self) -> Result<Vec<OnCallResource>> {
        self.paginate_all("/on-calls", &[]).await
    }

    pub async fn get_oncall(&self, id: &str) -> Result<OnCallDetailResponse> {
        let path = format!("/on-calls/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("API error ({}):\n{}", status, format_error_body(&body));
        }
        resp.json().await.context("Failed to parse response")
    }

    pub async fn create_oncall(&self, req: &CreateOnCallRequest) -> Result<OnCallResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/on-calls").json(req).send().await?) }).await?;
        super::parse_one(resp).await
    }

    pub async fn delete_oncall(&self, id: &str) -> Result<()> {
        let path = format!("/on-calls/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    pub async fn list_oncall_events(&self, id: &str) -> Result<Vec<OnCallEventResource>> {
        let path = format!("/on-calls/{id}/events");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("API error ({}):\n{}", status, format_error_body(&body));
        }
        let events: OnCallEventsResponse = resp.json().await.context("Failed to parse response")?;
        Ok(events.data)
    }

    pub async fn create_oncall_event(
        &self,
        id: &str,
        req: &CreateOnCallEventRequest,
    ) -> Result<()> {
        let path = format!("/on-calls/{id}/events");
        let resp = with_retry(|| async { Ok(self.post(&path).json(req).send().await?) }).await?;
        check_status(resp).await
    }
}
