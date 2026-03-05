use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct IncidentResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: IncidentAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct IncidentAttributes {
    pub name: Option<String>,
    pub url: Option<String>,
    pub http_method: Option<String>,
    pub cause: Option<String>,
    pub incident_group_id: Option<serde_json::Value>,
    pub started_at: Option<String>,
    pub acknowledged_at: Option<String>,
    pub acknowledged_by: Option<String>,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
    pub response_content: Option<String>,
    pub response_options: Option<String>,
    pub regions: Option<Vec<String>>,
    pub response_url: Option<String>,
    pub screenshot_url: Option<String>,
    pub escalation_policy_id: Option<serde_json::Value>,
    pub call: Option<bool>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub push: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct IncidentFilters {
    pub status: Option<String>,
    pub monitor_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateIncidentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requester_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TimelineEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: TimelineEventAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TimelineEventAttributes {
    pub event_type: Option<String>,
    pub started_at: Option<String>,
    pub duration: Option<f64>,
    pub regions: Option<Vec<String>>,
}
