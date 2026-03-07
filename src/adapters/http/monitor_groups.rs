use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{CreateMonitorGroupRequest, MonitorGroupResource, UpdateMonitorGroupRequest};

impl HttpClient {
    pub async fn list_monitor_groups(&self) -> Result<Vec<MonitorGroupResource>> {
        self.paginate_all("/monitor-groups", &[]).await
    }

    pub async fn get_monitor_group(&self, id: &str) -> Result<MonitorGroupResource> {
        let path = format!("/monitor-groups/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_monitor_group(
        &self,
        req: &CreateMonitorGroupRequest,
    ) -> Result<MonitorGroupResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/monitor-groups").json(req).send().await?) })
                .await?;
        parse_one(resp).await
    }

    pub async fn update_monitor_group(
        &self,
        id: &str,
        req: &UpdateMonitorGroupRequest,
    ) -> Result<MonitorGroupResource> {
        let path = format!("/monitor-groups/{id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_monitor_group(&self, id: &str) -> Result<()> {
        let path = format!("/monitor-groups/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }
}
