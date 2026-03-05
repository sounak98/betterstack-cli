use std::time::Duration;

use anyhow::Result;
use reqwest::StatusCode;

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 200;

/// Execute an async HTTP operation with retry on 429 and 5xx errors.
pub async fn with_retry<F, Fut>(f: F) -> Result<reqwest::Response>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<reqwest::Response>>,
{
    let mut attempts = 0;

    loop {
        let response = f().await?;
        let status = response.status();

        if status.is_success() || status.is_redirection() {
            return Ok(response);
        }

        let retryable = status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error();

        if retryable && attempts < MAX_RETRIES {
            let delay = Duration::from_millis(BASE_DELAY_MS * 2u64.pow(attempts));
            tokio::time::sleep(delay).await;
            attempts += 1;
            continue;
        }

        // Non-retryable or exhausted retries: return the response as-is
        // and let the caller handle the error status.
        return Ok(response);
    }
}
