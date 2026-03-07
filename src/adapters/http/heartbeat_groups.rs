use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{
    CreateHeartbeatGroupRequest, HeartbeatGroupResource, UpdateHeartbeatGroupRequest,
};

impl HttpClient {
    pub async fn list_heartbeat_groups(&self) -> Result<Vec<HeartbeatGroupResource>> {
        self.paginate_all("/heartbeat-groups", &[]).await
    }

    pub async fn get_heartbeat_group(&self, id: &str) -> Result<HeartbeatGroupResource> {
        let path = format!("/heartbeat-groups/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_heartbeat_group(
        &self,
        req: &CreateHeartbeatGroupRequest,
    ) -> Result<HeartbeatGroupResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/heartbeat-groups").json(req).send().await?) })
                .await?;
        parse_one(resp).await
    }

    pub async fn update_heartbeat_group(
        &self,
        id: &str,
        req: &UpdateHeartbeatGroupRequest,
    ) -> Result<HeartbeatGroupResource> {
        let path = format!("/heartbeat-groups/{id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_heartbeat_group(&self, id: &str) -> Result<()> {
        let path = format!("/heartbeat-groups/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }
}
