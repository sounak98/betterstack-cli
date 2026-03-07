use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MonitorGroupResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: MonitorGroupAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MonitorGroupAttributes {
    pub name: Option<String>,
    pub sort_index: Option<serde_json::Value>,
    /// Read-only. Derived from whether all monitors in the group are paused.
    pub paused: Option<bool>,
    pub team_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateMonitorGroupRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMonitorGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<u64>,
}
