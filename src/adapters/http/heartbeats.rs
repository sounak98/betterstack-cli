use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{
    CreateHeartbeatRequest, HeartbeatResource, SlaResource, UpdateHeartbeatRequest,
};

impl HttpClient {
    pub async fn list_heartbeats(&self) -> Result<Vec<HeartbeatResource>> {
        self.paginate_all("/heartbeats", &[]).await
    }

    pub async fn get_heartbeat(&self, id: &str) -> Result<HeartbeatResource> {
        let path = format!("/heartbeats/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_heartbeat(
        &self,
        req: &CreateHeartbeatRequest,
    ) -> Result<HeartbeatResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/heartbeats").json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_heartbeat(
        &self,
        id: &str,
        req: &UpdateHeartbeatRequest,
    ) -> Result<HeartbeatResource> {
        let path = format!("/heartbeats/{id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_heartbeat(&self, id: &str) -> Result<()> {
        let path = format!("/heartbeats/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    pub async fn heartbeat_availability(
        &self,
        id: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<SlaResource> {
        let path = format!("/heartbeats/{id}/sla");
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
