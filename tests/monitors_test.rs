mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

#[tokio::test]
async fn list_monitors_returns_data() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/monitors"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("monitors_list.json")))
        .mount(&mock_server)
        .await;

    let client = bs_cli::adapters::http::HttpClient::new(
        &format!("{}/api/v2", mock_server.uri()),
        "test-token",
    );
    let filters = bs_cli::types::MonitorFilters::default();

    let monitors = client.list_monitors(&filters).await.unwrap();

    assert_eq!(monitors.len(), 2);
    assert_eq!(monitors[0].id, "123");
    assert_eq!(
        monitors[0].attributes.pronounceable_name.as_deref(),
        Some("Example Site")
    );
    assert_eq!(monitors[0].attributes.status.as_deref(), Some("up"));
    assert_eq!(monitors[1].id, "456");
    assert_eq!(monitors[1].attributes.status.as_deref(), Some("down"));
}

#[tokio::test]
async fn get_monitor_returns_single() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/monitors/123"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("monitor_get.json")))
        .mount(&mock_server)
        .await;

    let client = bs_cli::adapters::http::HttpClient::new(
        &format!("{}/api/v2", mock_server.uri()),
        "test-token",
    );

    let monitor = client.get_monitor("123").await.unwrap();

    assert_eq!(monitor.id, "123");
    assert_eq!(
        monitor.attributes.url.as_deref(),
        Some("https://example.com")
    );
}

#[tokio::test]
async fn list_monitors_handles_pagination() {
    let mock_server = MockServer::start().await;
    let base_uri = mock_server.uri();

    let page1 = serde_json::json!({
        "data": [{
            "id": "1",
            "type": "monitor",
            "attributes": {
                "url": "https://one.com",
                "pronounceable_name": "One",
                "monitor_type": "status",
                "status": "up",
                "check_frequency": 30
            }
        }],
        "pagination": {
            "first": format!("{}/api/v2/monitors?page=1", base_uri),
            "last": format!("{}/api/v2/monitors?page=2", base_uri),
            "prev": null,
            "next": format!("{}/api/v2/monitors?page=2", base_uri)
        }
    });

    let page2 = serde_json::json!({
        "data": [{
            "id": "2",
            "type": "monitor",
            "attributes": {
                "url": "https://two.com",
                "pronounceable_name": "Two",
                "monitor_type": "status",
                "status": "down",
                "check_frequency": 60
            }
        }],
        "pagination": {
            "first": format!("{}/api/v2/monitors?page=1", base_uri),
            "last": format!("{}/api/v2/monitors?page=2", base_uri),
            "prev": format!("{}/api/v2/monitors?page=1", base_uri),
            "next": null
        }
    });

    // First page (no query param)
    Mock::given(method("GET"))
        .and(path("/api/v2/monitors"))
        .and(wiremock::matchers::query_param_is_missing("page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page1))
        .mount(&mock_server)
        .await;

    // Second page
    Mock::given(method("GET"))
        .and(path("/api/v2/monitors"))
        .and(wiremock::matchers::query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page2))
        .mount(&mock_server)
        .await;

    let client = bs_cli::adapters::http::HttpClient::new(
        &format!("{}/api/v2", mock_server.uri()),
        "test-token",
    );
    let filters = bs_cli::types::MonitorFilters::default();

    let monitors = client.list_monitors(&filters).await.unwrap();

    assert_eq!(monitors.len(), 2);
    assert_eq!(monitors[0].id, "1");
    assert_eq!(monitors[1].id, "2");
}

#[tokio::test]
async fn api_error_returns_descriptive_message() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/monitors/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"errors":"Not found"}"#))
        .mount(&mock_server)
        .await;

    let client = bs_cli::adapters::http::HttpClient::new(
        &format!("{}/api/v2", mock_server.uri()),
        "test-token",
    );

    let result = client.get_monitor("999").await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("404"), "Error should contain status code");
}
