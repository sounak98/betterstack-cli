use anyhow::Result;

use super::retry::with_retry;
use super::{HttpClient, check_status, parse_one};
use crate::types::{
    CreateStatusPageItemRequest, CreateStatusPageRequest, CreateStatusPageSectionRequest,
    CreateStatusReportRequest, CreateStatusUpdateRequest, StatusPageItemResource,
    StatusPageResource, StatusPageSectionResource, StatusReportResource, StatusUpdateResource,
    UpdateStatusPageItemRequest, UpdateStatusPageRequest, UpdateStatusPageSectionRequest,
    UpdateStatusReportRequest,
};

impl HttpClient {
    // --- Status Pages ---

    pub async fn list_status_pages(&self) -> Result<Vec<StatusPageResource>> {
        self.paginate_all("/status-pages", &[]).await
    }

    pub async fn get_status_page(&self, id: &str) -> Result<StatusPageResource> {
        let path = format!("/status-pages/{id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_status_page(
        &self,
        req: &CreateStatusPageRequest,
    ) -> Result<StatusPageResource> {
        let resp =
            with_retry(|| async { Ok(self.post("/status-pages").json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_status_page(
        &self,
        id: &str,
        req: &UpdateStatusPageRequest,
    ) -> Result<StatusPageResource> {
        let path = format!("/status-pages/{id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_status_page(&self, id: &str) -> Result<()> {
        let path = format!("/status-pages/{id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    // --- Sections ---

    pub async fn list_status_page_sections(
        &self,
        page_id: &str,
    ) -> Result<Vec<StatusPageSectionResource>> {
        let path = format!("/status-pages/{page_id}/sections");
        self.paginate_all(&path, &[]).await
    }

    pub async fn get_status_page_section(
        &self,
        page_id: &str,
        section_id: &str,
    ) -> Result<StatusPageSectionResource> {
        let path = format!("/status-pages/{page_id}/sections/{section_id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_status_page_section(
        &self,
        page_id: &str,
        req: &CreateStatusPageSectionRequest,
    ) -> Result<StatusPageSectionResource> {
        let path = format!("/status-pages/{page_id}/sections");
        let resp = with_retry(|| async { Ok(self.post(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_status_page_section(
        &self,
        page_id: &str,
        section_id: &str,
        req: &UpdateStatusPageSectionRequest,
    ) -> Result<StatusPageSectionResource> {
        let path = format!("/status-pages/{page_id}/sections/{section_id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_status_page_section(&self, page_id: &str, section_id: &str) -> Result<()> {
        let path = format!("/status-pages/{page_id}/sections/{section_id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    // --- Resources (items on status page) ---

    pub async fn list_status_page_resources(
        &self,
        page_id: &str,
    ) -> Result<Vec<StatusPageItemResource>> {
        let path = format!("/status-pages/{page_id}/resources");
        self.paginate_all(&path, &[]).await
    }

    pub async fn get_status_page_resource(
        &self,
        page_id: &str,
        resource_id: &str,
    ) -> Result<StatusPageItemResource> {
        let path = format!("/status-pages/{page_id}/resources/{resource_id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_status_page_resource(
        &self,
        page_id: &str,
        req: &CreateStatusPageItemRequest,
    ) -> Result<StatusPageItemResource> {
        let path = format!("/status-pages/{page_id}/resources");
        let resp = with_retry(|| async { Ok(self.post(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_status_page_resource(
        &self,
        page_id: &str,
        resource_id: &str,
        req: &UpdateStatusPageItemRequest,
    ) -> Result<StatusPageItemResource> {
        let path = format!("/status-pages/{page_id}/resources/{resource_id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_status_page_resource(
        &self,
        page_id: &str,
        resource_id: &str,
    ) -> Result<()> {
        let path = format!("/status-pages/{page_id}/resources/{resource_id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    // --- Reports ---

    pub async fn list_status_reports(&self, page_id: &str) -> Result<Vec<StatusReportResource>> {
        let path = format!("/status-pages/{page_id}/status-reports");
        self.paginate_all(&path, &[]).await
    }

    pub async fn get_status_report(
        &self,
        page_id: &str,
        report_id: &str,
    ) -> Result<StatusReportResource> {
        let path = format!("/status-pages/{page_id}/status-reports/{report_id}");
        let resp = with_retry(|| async { Ok(self.get(&path).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn create_status_report(
        &self,
        page_id: &str,
        req: &CreateStatusReportRequest,
    ) -> Result<StatusReportResource> {
        let path = format!("/status-pages/{page_id}/status-reports");
        let resp = with_retry(|| async { Ok(self.post(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn update_status_report(
        &self,
        page_id: &str,
        report_id: &str,
        req: &UpdateStatusReportRequest,
    ) -> Result<StatusReportResource> {
        let path = format!("/status-pages/{page_id}/status-reports/{report_id}");
        let resp = with_retry(|| async { Ok(self.patch(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }

    pub async fn delete_status_report(&self, page_id: &str, report_id: &str) -> Result<()> {
        let path = format!("/status-pages/{page_id}/status-reports/{report_id}");
        let resp = with_retry(|| async { Ok(self.delete_req(&path).send().await?) }).await?;
        check_status(resp).await
    }

    // --- Report Updates ---

    pub async fn list_status_updates(
        &self,
        page_id: &str,
        report_id: &str,
    ) -> Result<Vec<StatusUpdateResource>> {
        let path = format!("/status-pages/{page_id}/status-reports/{report_id}/status-updates");
        self.paginate_all(&path, &[]).await
    }

    pub async fn create_status_update(
        &self,
        page_id: &str,
        report_id: &str,
        req: &CreateStatusUpdateRequest,
    ) -> Result<StatusUpdateResource> {
        let path = format!("/status-pages/{page_id}/status-reports/{report_id}/status-updates");
        let resp = with_retry(|| async { Ok(self.post(&path).json(req).send().await?) }).await?;
        parse_one(resp).await
    }
}
