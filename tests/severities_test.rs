mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

fn client(base: &str) -> bs_cli::adapters::http::HttpClient {
    bs_cli::adapters::http::HttpClient::new(&format!("{base}/api/v2"), "test-token")
}

#[tokio::test]
async fn list_severities_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/urgencies"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("severities_list.json")),
        )
        .mount(&mock)
        .await;

    let severities = client(&mock.uri()).list_severities().await.unwrap();

    assert_eq!(severities.len(), 2);
    assert_eq!(severities[0].id, "400");
    assert_eq!(
        severities[0].attributes.name.as_deref(),
        Some("SEV1 - Critical")
    );
    assert_eq!(severities[0].attributes.email, Some(true));
    assert_eq!(severities[0].attributes.sms, Some(true));
    assert_eq!(severities[0].attributes.call, Some(true));
    assert_eq!(severities[0].attributes.push, Some(true));
    assert_eq!(severities[0].attributes.critical_alert, Some(true));

    assert_eq!(severities[1].id, "401");
    assert_eq!(severities[1].attributes.name.as_deref(), Some("SEV4 - Low"));
    assert_eq!(severities[1].attributes.sms, Some(false));
    assert_eq!(severities[1].attributes.call, Some(false));
    assert_eq!(severities[1].attributes.critical_alert, Some(false));
}

#[tokio::test]
async fn get_severity_returns_single() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/urgencies/400"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("severity_get.json")))
        .mount(&mock)
        .await;

    let sev = client(&mock.uri()).get_severity("400").await.unwrap();

    assert_eq!(sev.id, "400");
    assert_eq!(sev.attributes.name.as_deref(), Some("SEV1 - Critical"));
    assert_eq!(sev.attributes.email, Some(true));
    assert_eq!(sev.attributes.critical_alert, Some(true));
    assert_eq!(sev.attributes.team_name.as_deref(), Some("Engineering"));
}

#[tokio::test]
async fn create_severity_sends_request() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/urgencies"))
        .respond_with(ResponseTemplate::new(201).set_body_json(load_fixture("severity_get.json")))
        .mount(&mock)
        .await;

    let req = bs_cli::types::CreateSeverityRequest {
        name: "SEV1 - Critical".to_string(),
        email: true,
        sms: true,
        call: true,
        push: true,
        critical_alert: Some(true),
    };
    let sev = client(&mock.uri()).create_severity(&req).await.unwrap();

    assert_eq!(sev.id, "400");
    assert_eq!(sev.attributes.name.as_deref(), Some("SEV1 - Critical"));
}

#[tokio::test]
async fn update_severity_sends_patch() {
    let mock = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/v2/urgencies/400"))
        .respond_with(ResponseTemplate::new(200).set_body_json(load_fixture("severity_get.json")))
        .mount(&mock)
        .await;

    let req = bs_cli::types::UpdateSeverityRequest {
        name: Some("Updated Severity".to_string()),
        email: Some(true),
        sms: Some(false),
        call: Some(false),
        push: Some(false),
        critical_alert: Some(false),
    };
    let sev = client(&mock.uri())
        .update_severity("400", &req)
        .await
        .unwrap();

    assert_eq!(sev.id, "400");
}

#[tokio::test]
async fn delete_severity_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/urgencies/400"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_severity("400").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_severity_404_returns_error() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/urgencies/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"errors":"Not found"}"#))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_severity("999").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("404"));
}
