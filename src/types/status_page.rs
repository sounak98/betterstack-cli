use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusPageResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: StatusPageAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusPageAttributes {
    pub company_name: Option<String>,
    pub company_url: Option<String>,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub timezone: Option<String>,
    pub theme: Option<String>,
    pub layout: Option<String>,
    pub aggregate_state: Option<String>,
    pub password_enabled: Option<bool>,
    pub subscribable: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub team_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStatusPageRequest {
    pub company_name: String,
    pub subdomain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribable: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusPageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribable: Option<bool>,
}

// --- Sections ---

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusPageSectionResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: StatusPageSectionAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusPageSectionAttributes {
    pub name: Option<String>,
    pub position: Option<u64>,
    pub status_page_id: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStatusPageSectionRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusPageSectionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

// --- Resources (monitors on status page) ---

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusPageItemResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: StatusPageItemAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusPageItemAttributes {
    pub status_page_section_id: Option<serde_json::Value>,
    pub resource_id: Option<serde_json::Value>,
    pub resource_type: Option<String>,
    pub public_name: Option<String>,
    pub status: Option<String>,
    pub availability: Option<f64>,
    pub widget_type: Option<String>,
    pub position: Option<u64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStatusPageItemRequest {
    pub resource_id: u64,
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_page_section_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusPageItemRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_page_section_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

// --- Reports ---

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusReportResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: StatusReportAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusReportAttributes {
    pub title: Option<String>,
    pub report_type: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub aggregate_state: Option<String>,
    pub status_page_id: Option<serde_json::Value>,
    pub affected_resources: Option<Vec<AffectedResource>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AffectedResource {
    pub status_page_resource_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStatusReportRequest {
    pub title: String,
    pub report_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_resources: Option<Vec<AffectedResource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusReportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<String>,
}

// --- Report Updates ---

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusUpdateResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: StatusUpdateAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusUpdateAttributes {
    pub message: Option<String>,
    pub published_at: Option<String>,
    pub notify_subscribers: Option<bool>,
    pub status_report_id: Option<serde_json::Value>,
    pub affected_resources: Option<Vec<AffectedResource>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStatusUpdateRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_subscribers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_resources: Option<Vec<AffectedResource>>,
}
