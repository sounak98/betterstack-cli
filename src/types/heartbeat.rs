use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: HeartbeatAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatAttributes {
    pub url: Option<String>,
    pub name: Option<String>,
    pub period: Option<u64>,
    pub grace: Option<u64>,
    pub call: Option<bool>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub push: Option<bool>,
    pub critical_alert: Option<bool>,
    pub team_wait: Option<u64>,
    pub heartbeat_group_id: Option<serde_json::Value>,
    pub sort_index: Option<serde_json::Value>,
    pub status: Option<String>,
    pub paused_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub team_name: Option<String>,
    pub policy_id: Option<serde_json::Value>,
    pub maintenance_from: Option<String>,
    pub maintenance_to: Option<String>,
    pub maintenance_timezone: Option<String>,
    pub maintenance_days: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateHeartbeatRequest {
    pub name: String,
    pub period: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_alert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_wait: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateHeartbeatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_alert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_wait: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct HeartbeatFilters {
    pub status: Option<String>,
}
