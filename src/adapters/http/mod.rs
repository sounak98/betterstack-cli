pub mod pagination;
pub mod retry;
pub mod uptime;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

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

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.url(path)).headers(self.headers())
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(self.url(path)).headers(self.headers())
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.patch(self.url(path)).headers(self.headers())
    }

    pub fn delete_req(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.delete(self.url(path)).headers(self.headers())
    }

    pub fn get_absolute(&self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url).headers(self.headers())
    }
}
