mod common;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::load_fixture;

fn client(base: &str) -> betterstack_cli::adapters::http::HttpClient {
    betterstack_cli::adapters::http::HttpClient::new(&format!("{base}/api/v2"), "test-token")
}

// --- Status Pages ---

#[tokio::test]
async fn list_status_pages_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/status-pages"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("status_pages_list.json")),
        )
        .mount(&mock)
        .await;

    let pages = client(&mock.uri()).list_status_pages().await.unwrap();

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].id, "100");
    assert_eq!(
        pages[0].attributes.company_name.as_deref(),
        Some("Acme Corp")
    );
    assert_eq!(
        pages[0].attributes.aggregate_state.as_deref(),
        Some("operational")
    );
    assert_eq!(pages[1].id, "101");
    assert_eq!(
        pages[1].attributes.aggregate_state.as_deref(),
        Some("downtime")
    );
}

#[tokio::test]
async fn get_status_page_returns_single() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/status-pages/100"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("status_page_get.json")),
        )
        .mount(&mock)
        .await;

    let page = client(&mock.uri()).get_status_page("100").await.unwrap();

    assert_eq!(page.id, "100");
    assert_eq!(page.attributes.company_name.as_deref(), Some("Acme Corp"));
    assert_eq!(page.attributes.subdomain.as_deref(), Some("acme"));
    assert_eq!(page.attributes.subscribable, Some(true));
}

#[tokio::test]
async fn create_status_page_sends_request() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/status-pages"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(load_fixture("status_page_get.json")),
        )
        .mount(&mock)
        .await;

    let req = betterstack_cli::types::CreateStatusPageRequest {
        company_name: "Acme Corp".to_string(),
        subdomain: "acme".to_string(),
        company_url: Some("https://acme.com".to_string()),
        custom_domain: None,
        timezone: Some("UTC".to_string()),
        theme: Some("light".to_string()),
        subscribable: Some(true),
    };
    let page = client(&mock.uri()).create_status_page(&req).await.unwrap();

    assert_eq!(page.id, "100");
    assert_eq!(page.attributes.company_name.as_deref(), Some("Acme Corp"));
}

#[tokio::test]
async fn delete_status_page_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/status-pages/100"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_status_page("100").await;
    assert!(result.is_ok());
}

// --- Sections ---

#[tokio::test]
async fn list_status_page_sections_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/status-pages/100/sections"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(load_fixture("status_page_sections_list.json")),
        )
        .mount(&mock)
        .await;

    let sections = client(&mock.uri())
        .list_status_page_sections("100")
        .await
        .unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].id, "200");
    assert_eq!(
        sections[0].attributes.name.as_deref(),
        Some("Core Services")
    );
    assert_eq!(sections[1].id, "201");
    assert_eq!(
        sections[1].attributes.name.as_deref(),
        Some("API Endpoints")
    );
}

#[tokio::test]
async fn delete_status_page_section_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/status-pages/100/sections/200"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri())
        .delete_status_page_section("100", "200")
        .await;
    assert!(result.is_ok());
}

// --- Resources ---

#[tokio::test]
async fn list_status_page_resources_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/status-pages/100/resources"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(load_fixture("status_page_resources_list.json")),
        )
        .mount(&mock)
        .await;

    let resources = client(&mock.uri())
        .list_status_page_resources("100")
        .await
        .unwrap();

    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0].id, "300");
    assert_eq!(
        resources[0].attributes.public_name.as_deref(),
        Some("Website")
    );
    assert_eq!(resources[0].attributes.availability, Some(0.9995));
    assert_eq!(resources[1].id, "301");
    assert_eq!(resources[1].attributes.status.as_deref(), Some("downtime"));
}

// --- Reports ---

#[tokio::test]
async fn list_status_reports_returns_data() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/status-pages/100/status-reports"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("status_reports_list.json")),
        )
        .mount(&mock)
        .await;

    let reports = client(&mock.uri())
        .list_status_reports("100")
        .await
        .unwrap();

    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].id, "400");
    assert_eq!(
        reports[0].attributes.title.as_deref(),
        Some("Database maintenance")
    );
    assert_eq!(
        reports[0].attributes.report_type.as_deref(),
        Some("maintenance")
    );
    assert_eq!(reports[1].id, "401");
    assert_eq!(
        reports[1].attributes.aggregate_state.as_deref(),
        Some("degraded")
    );
}

#[tokio::test]
async fn get_status_report_returns_single() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/status-pages/100/status-reports/401"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(load_fixture("status_report_get.json")),
        )
        .mount(&mock)
        .await;

    let report = client(&mock.uri())
        .get_status_report("100", "401")
        .await
        .unwrap();

    assert_eq!(report.id, "401");
    assert_eq!(
        report.attributes.title.as_deref(),
        Some("API degraded performance")
    );
    assert!(report.attributes.affected_resources.is_some());
    let affected = report.attributes.affected_resources.as_ref().unwrap();
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].status_page_resource_id.as_deref(), Some("301"));
}

#[tokio::test]
async fn delete_status_report_succeeds() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v2/status-pages/100/status-reports/401"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = client(&mock.uri()).delete_status_report("100", "401").await;
    assert!(result.is_ok());
}
