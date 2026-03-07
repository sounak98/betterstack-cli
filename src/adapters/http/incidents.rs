use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{CreateIncidentRequest, IncidentFilters, IncidentResource, TimelineEvent};

impl HttpClient {
    pub async fn list_incidents(&self, filters: &IncidentFilters) -> Result<Vec<IncidentResource>> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(ref status) = filters.status {
            params.push(("status", status.as_str()));
        }
        if let Some(ref monitor_id) = filters.monitor_id {
            params.push(("monitor_id", monitor_id.as_str()));
        }
        if let Some(ref from) = filters.from {
            params.push(("from", from.as_str()));
        }
        if let Some(ref to) = filters.to {
            params.push(("to", to.as_str()));
        }

        self.paginate_all_v3("/incidents", &params).await
    }

    pub async fn get_incident(&self, id: &str) -> Result<IncidentResource> {
        let path = format!("/incidents/{id}");
        let resp = with_retry(|| async { Ok(self.get_v3(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_incident(&self, req: &CreateIncidentRequest) -> Result<IncidentResource> {
        let resp =
            with_retry(|| async { Ok(self.post_v3("/incidents").json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn acknowledge_incident(
        &self,
        id: &str,
        by: Option<&str>,
    ) -> Result<IncidentResource> {
        let path = format!("/incidents/{id}/acknowledge");
        let resp = with_retry(|| async {
            let mut req = self.post_v3(&path);
            if let Some(email) = by {
                req = req.json(&serde_json::json!({ "acknowledged_by": email }));
            }
            Ok(req.send().await?)
        })
        .await?;
        parse_one(resp).await
    }

    pub async fn resolve_incident(&self, id: &str, by: Option<&str>) -> Result<IncidentResource> {
        let path = format!("/incidents/{id}/resolve");
        let resp = with_retry(|| async {
            let mut req = self.post_v3(&path);
            if let Some(email) = by {
                req = req.json(&serde_json::json!({ "resolved_by": email }));
            }
            Ok(req.send().await?)
        })
        .await?;
        parse_one(resp).await
    }

    pub async fn escalate_incident(&self, id: &str) -> Result<IncidentResource> {
        let path = format!("/incidents/{id}/escalate");
        let resp = with_retry(|| async { Ok(self.post_v3(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_incident(&self, id: &str) -> Result<()> {
        let path = format!("/incidents/{id}");
        let resp = with_retry(|| async { Ok(self.delete_v3(&path).send().await?) }).await?;
        check_status(resp).await
    }

    pub async fn incident_timeline(&self, id: &str) -> Result<Vec<TimelineEvent>> {
        let path = format!("/incidents/{id}/timeline");
        self.paginate_all_v3(&path, &[]).await
    }
}
