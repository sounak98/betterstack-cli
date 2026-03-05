use anyhow::{Result, bail};
use serde::Deserialize;

use super::HttpClient;
use super::retry::with_retry;

#[derive(Debug, Deserialize)]
struct ConnectionResponse {
    data: ConnectionData,
}

#[derive(Debug, Deserialize)]
struct ConnectionData {
    attributes: ConnectionAttributes,
}

#[derive(Debug, Deserialize)]
struct ConnectionAttributes {
    host: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

pub struct ConnectionCredentials {
    pub host: String,
    pub username: String,
    pub password: String,
}

impl HttpClient {
    pub async fn create_sql_connection(&self) -> Result<ConnectionCredentials> {
        let body = serde_json::json!({
            "client_type": "clickhouse"
        });

        let resp = with_retry(|| async { Ok(self.post("/connections").json(&body).send().await?) })
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("Failed to create SQL connection ({}): {}", status, body);
        }

        let parsed: ConnectionResponse = resp.json().await?;
        let attrs = parsed.data.attributes;

        let host = attrs
            .host
            .ok_or_else(|| anyhow::anyhow!("No host in connection response"))?;
        let username = attrs
            .username
            .ok_or_else(|| anyhow::anyhow!("No username in connection response"))?;
        let password = attrs
            .password
            .ok_or_else(|| anyhow::anyhow!("No password in connection response"))?;

        Ok(ConnectionCredentials {
            host,
            username,
            password,
        })
    }
}
