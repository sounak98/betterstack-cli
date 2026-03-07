mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

fn client(base: &str) -> bs_cli::adapters::http::HttpClient {
    bs_cli::adapters::http::HttpClient::new(&format!("{base}/api/v2"), "test-token")
}

#[tokio::test]
async fn list_heartbeats_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/heartbeats"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("heartbeats_list.json")))
        .mount(&mock)
        .await;

    let heartbeats = client(&mock.uri()).list_heartbeats().await.unwrap();

    assert_eq!(heartbeats.len(), 2);
    assert_eq!(heartbeats[0].id, "100");
    assert_eq!(heartbeats[0].attributes.name.as_deref(), Some("Nightly Backup"));
    assert_eq!(heartbeats[0].attributes.period, Some(86400));
    assert_eq!(heartbeats[0].attributes.status.as_deref(), Some("up"));
    assert_eq!(heartbeats[1].id, "101");
    assert_eq!(heartbeats[1].attributes.name.as_deref(), Some("Queue Worker"));
    assert_eq!(heartbeats[1].attributes.status.as_deref(), Some("down"));
}

#[tokio::test]
async fn get_heartbeat_returns_single() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/heartbeats/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("heartbeat_get.json")))
        .mount(&mock)
        .await;

    let hb = client(&mock.uri()).get_heartbeat("100").await.unwrap();

    assert_eq!(hb.id, "100");
    assert_eq!(hb.attributes.name.as_deref(), Some("Nightly Backup"));
    assert_eq!(hb.attributes.period, Some(86400));
    assert_eq!(hb.attributes.grace, Some(3600));
    assert_eq!(hb.attributes.email, Some(true));
    assert_eq!(hb.attributes.call, Some(false));
}

#[tokio::test]
async fn create_heartbeat_sends_correct_body() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/heartbeats"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(load_fixture("heartbeat_create.json")),
        )
        .mount(&mock)
        .await;

    let req = bs_cli::types::CreateHeartbeatRequest {
        name: "Deploy Check".to_string(),
        period: 600,
        grace: Some(120),
        call: None,
        sms: None,
        email: Some(true),
        push: None,
        critical_alert: None,
        team_wait: None,
        heartbeat_group_id: None,
        sort_index: None,
        paused: None,
        policy_id: None,
    };
    let hb = client(&mock.uri()).create_heartbeat(&req).await.unwrap();

    assert_eq!(hb.id, "102");
    assert_eq!(hb.attributes.name.as_deref(), Some("Deploy Check"));
    assert_eq!(hb.attributes.status.as_deref(), Some("pending"));
}

#[tokio::test]
async fn delete_heartbeat_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/heartbeats/100"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_heartbeat("100").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_heartbeat_404_returns_error() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/heartbeats/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"errors":"Not found"}"#))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_heartbeat("999").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("404"));
}
