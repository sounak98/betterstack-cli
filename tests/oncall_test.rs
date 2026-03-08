mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

fn client(base: &str) -> betterstack_cli::adapters::http::HttpClient {
    betterstack_cli::adapters::http::HttpClient::new(&format!("{base}/api/v2"), "test-token")
}

#[tokio::test]
async fn list_oncalls_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/on-calls"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("oncalls_list.json")))
        .mount(&mock)
        .await;

    let calendars = client(&mock.uri()).list_oncalls().await.unwrap();

    assert_eq!(calendars.len(), 2);
    assert_eq!(calendars[0].id, "300");
    assert_eq!(
        calendars[0].attributes.name.as_deref(),
        Some("Primary On-Call")
    );
    assert_eq!(calendars[0].attributes.default_calendar, Some(true));
    assert_eq!(calendars[1].id, "301");
    assert_eq!(calendars[1].attributes.default_calendar, Some(false));
}

#[tokio::test]
async fn get_oncall_returns_detail_with_included_users() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/on-calls/300"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("oncall_get.json")))
        .mount(&mock)
        .await;

    let detail = client(&mock.uri()).get_oncall("300").await.unwrap();

    assert_eq!(detail.data.id, "300");
    assert_eq!(
        detail.data.attributes.name.as_deref(),
        Some("Primary On-Call")
    );

    let users = detail.included.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].attributes.first_name.as_deref(), Some("Alice"));
    assert_eq!(users[0].attributes.last_name.as_deref(), Some("Smith"));
    assert_eq!(
        users[0].attributes.email.as_deref(),
        Some("alice@example.com")
    );
}

#[tokio::test]
async fn list_oncall_events_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/on-calls/300/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("oncall_events.json")))
        .mount(&mock)
        .await;

    let events = client(&mock.uri()).list_oncall_events("300").await.unwrap();

    assert_eq!(events.len(), 3);
    assert_eq!(
        events[0].users.as_ref().unwrap(),
        &["alice@example.com".to_string()]
    );
    assert_eq!(events[0].is_override, Some(false));
    assert_eq!(events[2].is_override, Some(true));
    assert_eq!(events[2].users.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn create_oncall_returns_resource() {
    let mock = MockServer::start().await;

    let response = serde_json::json!({
        "data": {
            "id": "302",
            "type": "on_call_calendar",
            "attributes": {
                "name": "Weekend On-Call",
                "default_calendar": false,
                "team_name": "Engineering"
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v2/on-calls"))
        .respond_with(ResponseTemplate::new(201).set_body_json(response))
        .mount(&mock)
        .await;

    let req = betterstack_cli::types::CreateOnCallRequest {
        name: "Weekend On-Call".to_string(),
    };
    let cal = client(&mock.uri()).create_oncall(&req).await.unwrap();

    assert_eq!(cal.id, "302");
    assert_eq!(cal.attributes.name.as_deref(), Some("Weekend On-Call"));
}

#[tokio::test]
async fn delete_oncall_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/on-calls/301"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_oncall("301").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_oncall_event_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/on-calls/300/events"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock)
        .await;

    let req = betterstack_cli::types::CreateOnCallEventRequest {
        starts_at: "2026-03-10T09:00:00Z".to_string(),
        ends_at: "2026-03-10T17:00:00Z".to_string(),
        users: vec!["alice@example.com".to_string()],
        is_override: Some(true),
    };
    let result = client(&mock.uri()).create_oncall_event("300", &req).await;
    assert!(result.is_ok());
}
