use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatGroupResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: HeartbeatGroupAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatGroupAttributes {
    pub name: Option<String>,
    pub sort_index: Option<serde_json::Value>,
    pub paused: Option<bool>,
    pub team_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateHeartbeatGroupRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateHeartbeatGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<u64>,
}
