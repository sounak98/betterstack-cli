use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

use super::HttpClient;
use super::retry::with_retry;
use crate::types::PaginatedResponse;

impl HttpClient {
    /// Fetches all pages from a paginated endpoint, following `pagination.next` links.
    /// `path` is the initial path (e.g. "/monitors"), `query_params` are applied to the first request.
    pub async fn paginate_all<T: DeserializeOwned>(
        &self,
        path: &str,
        query_params: &[(&str, &str)],
    ) -> Result<Vec<T>> {
        let mut all = Vec::new();
        let mut next_url: Option<String> = None;

        loop {
            let resp = if let Some(ref url) = next_url {
                with_retry(|| async { Ok(self.get_absolute(url).send().await?) }).await?
            } else {
                let mut req = self.get(path);
                for (k, v) in query_params {
                    req = req.query(&[(*k, *v)]);
                }
                with_retry(|| async {
                    Ok(req
                        .try_clone()
                        .expect("request clone failed")
                        .send()
                        .await?)
                })
                .await?
            };

            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                bail!("API error ({}): {}", status, body);
            }

            let page: PaginatedResponse<T> = resp
                .json()
                .await
                .context("Failed to parse paginated response")?;

            all.extend(page.data);

            match page.pagination.and_then(|p| p.next) {
                Some(url) if !url.is_empty() => next_url = Some(url),
                _ => break,
            }
        }

        Ok(all)
    }
}
