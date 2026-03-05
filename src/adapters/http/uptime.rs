use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{
    CreateMonitorRequest, MonitorFilters, MonitorResource, ResponseTimesResource, SlaResource,
    UpdateMonitorRequest,
};

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

    pub async fn update_monitor(
        &self,
        id: &str,
        req: &UpdateMonitorRequest,
    ) -> Result<MonitorResource> {
        let path = format!("/monitors/{id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_monitor(&self, id: &str) -> Result<()> {
        let path = format!("/monitors/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    pub async fn monitor_sla(
        &self,
        id: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<SlaResource> {
        let path = format!("/monitors/{id}/sla");
        let resp = with_retry(|| async {
            let mut req = self.get(&path);
            if let Some(f) = from {
                req = req.query(&[("from", f)]);
            }
            if let Some(t) = to {
                req = req.query(&[("to", t)]);
            }
            Ok(req.send().await?)
        })
        .await?;
        parse_one(resp).await
    }

    pub async fn monitor_response_times(
        &self,
        id: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<ResponseTimesResource> {
        let path = format!("/monitors/{id}/response-times");
        let resp = with_retry(|| async {
            let mut req = self.get(&path);
            if let Some(f) = from {
                req = req.query(&[("from", f)]);
            }
            if let Some(t) = to {
                req = req.query(&[("to", t)]);
            }
            Ok(req.send().await?)
        })
        .await?;
        parse_one(resp).await
    }
}
