mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

fn client(base: &str) -> bs_cli::adapters::http::HttpClient {
    bs_cli::adapters::http::HttpClient::new(&format!("{base}/api/v2"), "test-token")
}

#[tokio::test]
async fn list_heartbeat_groups_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/heartbeat-groups"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("heartbeat_groups_list.json")),
        )
        .mount(&mock)
        .await;

    let groups = client(&mock.uri()).list_heartbeat_groups().await.unwrap();

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, "50");
    assert_eq!(groups[0].attributes.name.as_deref(), Some("Background Jobs"));
    assert_eq!(groups[0].attributes.paused, Some(false));
    assert_eq!(groups[1].id, "51");
    assert_eq!(groups[1].attributes.name.as_deref(), Some("Cron Jobs"));
    assert_eq!(groups[1].attributes.paused, Some(true));
}

#[tokio::test]
async fn get_heartbeat_group_returns_single() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/heartbeat-groups/50"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("heartbeat_group_get.json")),
        )
        .mount(&mock)
        .await;

    let group = client(&mock.uri()).get_heartbeat_group("50").await.unwrap();

    assert_eq!(group.id, "50");
    assert_eq!(group.attributes.name.as_deref(), Some("Background Jobs"));
    assert_eq!(group.attributes.paused, Some(false));
    assert_eq!(
        group.attributes.team_name.as_deref(),
        Some("Engineering")
    );
}

#[tokio::test]
async fn create_heartbeat_group_sends_request() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/heartbeat-groups"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(load_fixture("heartbeat_group_get.json")),
        )
        .mount(&mock)
        .await;

    let req = bs_cli::types::CreateHeartbeatGroupRequest {
        name: "Background Jobs".to_string(),
        paused: None,
        sort_index: None,
    };
    let group = client(&mock.uri())
        .create_heartbeat_group(&req)
        .await
        .unwrap();

    assert_eq!(group.id, "50");
    assert_eq!(group.attributes.name.as_deref(), Some("Background Jobs"));
}

#[tokio::test]
async fn delete_heartbeat_group_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/heartbeat-groups/50"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_heartbeat_group("50").await;
    assert!(result.is_ok());
}
