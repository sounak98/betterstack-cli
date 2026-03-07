use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{CreatePolicyRequest, PolicyResource, UpdatePolicyRequest};

impl HttpClient {
    pub async fn list_policies(&self) -> Result<Vec<PolicyResource>> {
        self.paginate_all_v3("/policies", &[]).await
    }

    pub async fn get_policy(&self, id: &str) -> Result<PolicyResource> {
        let path = format!("/policies/{id}");
        let resp = with_retry(|| async { Ok(self.get_v3(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_policy(&self, req: &CreatePolicyRequest) -> Result<PolicyResource> {
        let resp =
            with_retry(|| async { Ok(self.post_v3("/policies").json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_policy(
        &self,
        id: &str,
        req: &UpdatePolicyRequest,
    ) -> Result<PolicyResource> {
        let path = format!("/policies/{id}");
        let resp =
            with_retry(|| async { Ok(self.patch_v3(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_policy(&self, id: &str) -> Result<()> {
        let path = format!("/policies/{id}");
        let resp = with_retry(|| async { Ok(self.delete_v3(&path).send().await?) }).await?;
        check_status(resp).await
    }
}
