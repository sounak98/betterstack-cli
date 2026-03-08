mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

fn client(base: &str) -> betterstack_cli::adapters::http::HttpClient {
    betterstack_cli::adapters::http::HttpClient::new(&format!("{base}/api/v2"), "test-token")
}

#[tokio::test]
async fn list_policies_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/policies"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("policies_list.json")))
        .mount(&mock)
        .await;

    let policies = client(&mock.uri()).list_policies().await.unwrap();

    assert_eq!(policies.len(), 2);
    assert_eq!(policies[0].id, "200");
    assert_eq!(
        policies[0].attributes.name.as_deref(),
        Some("P1 - Production Outage")
    );
    assert_eq!(policies[0].attributes.repeat_count, Some(3));
    assert_eq!(policies[0].attributes.repeat_delay, Some(600));

    let steps = policies[0].attributes.steps.as_ref().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].step_type.as_deref(), Some("escalation"));
    assert_eq!(steps[0].wait_before, Some(0));
    assert_eq!(steps[1].wait_before, Some(300));

    assert_eq!(policies[1].id, "201");
    assert_eq!(
        policies[1].attributes.name.as_deref(),
        Some("P3 - Non-urgent")
    );
}

#[tokio::test]
async fn get_policy_returns_single_with_steps() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/policies/200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("policy_get.json")))
        .mount(&mock)
        .await;

    let policy = client(&mock.uri()).get_policy("200").await.unwrap();

    assert_eq!(policy.id, "200");
    assert_eq!(
        policy.attributes.name.as_deref(),
        Some("P1 - Production Outage")
    );
    assert_eq!(
        policy.attributes.incident_token.as_deref(),
        Some("tok_abc123")
    );

    let steps = policy.attributes.steps.as_ref().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].step_type.as_deref(), Some("escalation"));
    assert_eq!(steps[0].step_members.as_ref().unwrap().len(), 1);
    assert_eq!(steps[1].step_members.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn create_policy_sends_request() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v3/policies"))
        .respond_with(ResponseTemplate::new(201).set_body_json(load_fixture("policy_get.json")))
        .mount(&mock)
        .await;

    let step = betterstack_cli::types::PolicyStep {
        step_type: Some("escalation".to_string()),
        wait_before: Some(0),
        urgency_id: Some(1),
        step_members: Some(vec![serde_json::json!({"type": "current_on_call"})]),
        timezone: None,
        days: None,
        time_from: None,
        time_to: None,
        policy_id: None,
        policy_metadata_key: None,
    };
    let req = betterstack_cli::types::CreatePolicyRequest {
        name: "P1 - Production Outage".to_string(),
        steps: vec![step],
        repeat_count: Some(3),
        repeat_delay: Some(600),
        policy_group_id: None,
    };
    let policy = client(&mock.uri()).create_policy(&req).await.unwrap();

    assert_eq!(policy.id, "200");
}

#[tokio::test]
async fn update_policy_sends_patch() {
    let mock = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/v3/policies/200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("policy_get.json")))
        .mount(&mock)
        .await;

    let req = betterstack_cli::types::UpdatePolicyRequest {
        name: Some("Updated Policy".to_string()),
        steps: None,
        repeat_count: Some(5),
        repeat_delay: None,
        policy_group_id: None,
    };
    let policy = client(&mock.uri())
        .update_policy("200", &req)
        .await
        .unwrap();

    assert_eq!(policy.id, "200");
}

#[tokio::test]
async fn delete_policy_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/policies/200"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_policy("200").await;
    assert!(result.is_ok());
}
