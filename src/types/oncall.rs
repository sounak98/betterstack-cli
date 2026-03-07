use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: OnCallAttributes,
    pub relationships: Option<OnCallRelationships>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallAttributes {
    pub name: Option<String>,
    pub default_calendar: Option<bool>,
    pub team_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallRelationships {
    pub on_call_users: Option<OnCallUsersRelation>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallUsersRelation {
    pub data: Option<Vec<OnCallUserRef>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallUserRef {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub ref_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallIncludedUser {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub attributes: OnCallUserAttributes,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallUserAttributes {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone_numbers: Option<Vec<String>>,
}

/// Response for GET /on-calls/:id that includes user details.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallDetailResponse {
    pub data: OnCallResource,
    pub included: Option<Vec<OnCallIncludedUser>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallEventResource {
    pub id: Option<serde_json::Value>,
    pub users: Option<Vec<String>>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    #[serde(rename = "override")]
    pub is_override: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OnCallEventsResponse {
    pub data: Vec<OnCallEventResource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOnCallRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOnCallEventRequest {
    pub starts_at: String,
    pub ends_at: String,
    pub users: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "override")]
    pub is_override: Option<bool>,
}
