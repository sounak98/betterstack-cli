use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SeverityResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: SeverityAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SeverityAttributes {
    pub name: Option<String>,
    pub sms: Option<bool>,
    pub call: Option<bool>,
    pub email: Option<bool>,
    pub push: Option<bool>,
    pub critical_alert: Option<bool>,
    pub team_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSeverityRequest {
    pub name: String,
    pub sms: bool,
    pub call: bool,
    pub email: bool,
    pub push: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_alert: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateSeverityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_alert: Option<bool>,
}
