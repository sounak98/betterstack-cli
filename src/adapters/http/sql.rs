use anyhow::{Context, Result, bail};

use super::retry::with_retry;

pub struct SqlClient {
    client: reqwest::Client,
    base_url: String,
    username: String,
    password: String,
}

impl SqlClient {
    pub fn new(host: &str, username: &str, password: &str) -> Self {
        let base_url = if host.starts_with("https://") {
            host.to_string()
        } else {
            format!("https://{host}")
        };
        Self {
            client: reqwest::Client::new(),
            base_url,
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub async fn query(&self, sql: &str) -> Result<String> {
        let url = &self.base_url;
        let is_select = sql.trim_start().to_uppercase().starts_with("SELECT");
        let body = if is_select {
            format!("{sql} FORMAT JSONEachRow")
        } else {
            sql.to_string()
        };
        let resp = with_retry(|| {
            let body = body.clone();
            async move {
                Ok(self
                    .client
                    .post(url)
                    .basic_auth(&self.username, Some(&self.password))
                    .body(body)
                    .send()
                    .await?)
            }
        })
        .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!(
                "SQL query error ({}):\n{}",
                status,
                super::format_error_body(&body)
            );
        }

        resp.text().await.context("Failed to read SQL response")
    }

    pub async fn query_json(&self, sql: &str) -> Result<Vec<serde_json::Value>> {
        let raw = self.query(sql).await?;
        let mut rows = Vec::new();
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value: serde_json::Value =
                serde_json::from_str(line).context("Failed to parse SQL result row")?;
            rows.push(value);
        }
        Ok(rows)
    }
}
