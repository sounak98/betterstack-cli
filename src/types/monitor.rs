use serde::{Deserialize, Serialize};

// All fields are deserialized from the API. Some are only used in detail views.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MonitorResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: MonitorAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MonitorAttributes {
    pub url: Option<String>,
    pub pronounceable_name: Option<String>,
    pub monitor_type: Option<String>,
    pub status: Option<String>,
    pub check_frequency: Option<u32>,
    pub last_checked_at: Option<String>,
    pub paused_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub regions: Option<Vec<String>>,
    pub port: Option<serde_json::Value>,
    pub required_keyword: Option<String>,
    pub verify_ssl: Option<bool>,
    pub http_method: Option<String>,
    pub request_timeout: Option<u32>,
    pub recovery_period: Option<u32>,
    pub confirmation_period: Option<u32>,
    pub expected_status_codes: Option<Vec<u16>>,
    pub call: Option<bool>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub push: Option<bool>,
    pub team_name: Option<String>,
    pub maintenance_from: Option<String>,
    pub maintenance_to: Option<String>,
    pub maintenance_timezone: Option<String>,
    pub maintenance_days: Option<Vec<String>>,
    pub policy_id: Option<serde_json::Value>,
    pub monitor_group_id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateMonitorRequest {
    pub url: String,
    pub pronounceable_name: String,
    pub monitor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_frequency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_ssl: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_status_codes: Option<Vec<u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_period: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_period: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct MonitorFilters {
    pub status: Option<String>,
    pub monitor_type: Option<String>,
    pub url: Option<String>,
    pub pronounceable_name: Option<String>,
}
