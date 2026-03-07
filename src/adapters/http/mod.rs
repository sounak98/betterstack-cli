pub mod connections;
pub mod incidents;
pub mod pagination;
pub mod retry;
pub mod sql;
pub mod telemetry;
pub mod uptime;

use anyhow::{Context, Result, bail};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::types::SingleResponse;

pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

impl HttpClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    pub fn uptime(token: &str) -> Self {
        Self::new("https://uptime.betterstack.com/api/v2", token)
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", self.token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).expect("invalid token"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn url_v3(&self, path: &str) -> String {
        let v3_base = self.base_url.replace("/api/v2", "/api/v3");
        format!("{}{}", v3_base, path)
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.url(path)).headers(self.headers())
    }

    pub fn get_v3(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.url_v3(path)).headers(self.headers())
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(self.url(path)).headers(self.headers())
    }

    pub fn post_v3(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(self.url_v3(path)).headers(self.headers())
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.patch(self.url(path)).headers(self.headers())
    }

    pub fn delete_req(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.delete(self.url(path)).headers(self.headers())
    }

    pub fn delete_v3(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(self.url_v3(path))
            .headers(self.headers())
    }

    pub fn get_absolute(&self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url).headers(self.headers())
    }
}

/// Format an API error body, pretty-printing JSON if possible.
pub(super) fn format_error_body(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string())
    } else {
        body.to_string()
    }
}

/// Parse a single-resource response, or return an API error.
pub async fn parse_one<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("API error ({}):\n{}", status, format_error_body(&body));
    }
    let single: SingleResponse<T> = resp.json().await.context("Failed to parse response")?;
    Ok(single.data)
}

/// Check response status and return an error with body if not successful.
pub async fn check_status(resp: reqwest::Response) -> Result<()> {
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("API error ({}):\n{}", status, format_error_body(&body));
    }
    Ok(())
}
