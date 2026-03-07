use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PolicyResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: PolicyAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PolicyAttributes {
    pub name: Option<String>,
    pub repeat_count: Option<u64>,
    pub repeat_delay: Option<u64>,
    pub incident_token: Option<String>,
    pub policy_group_id: Option<serde_json::Value>,
    pub team_name: Option<String>,
    pub steps: Option<Vec<PolicyStep>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PolicyStep {
    #[serde(rename = "type")]
    pub step_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urgency_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_members: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_metadata_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub steps: Vec<PolicyStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_delay: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_group_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatePolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<PolicyStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_delay: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_group_id: Option<u64>,
}
