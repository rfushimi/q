use std::time::Duration;
use backoff::{ExponentialBackoff, backoff::Backoff};
use super::{CoreError, CoreResult};

pub async fn with_retry<T, F, Fut>(
    mut f: F,
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> CoreResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = CoreResult<T>>,
{
    let mut backoff = ExponentialBackoff {
        initial_interval: initial_delay,
        max_interval: max_delay,
        multiplier: 2.0,
        max_elapsed_time: None,
        ..ExponentialBackoff::default()
    };

    let mut attempt = 0;
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempt += 1;
                if attempt >= max_retries || !should_retry(&err) {
                    return Err(err);
                }

                if let Some(delay) = backoff.next_backoff() {
                    tokio::time::sleep(delay).await;
                } else {
                    return Err(CoreError::Retry(format!(
                        "Max retries ({}) exceeded",
                        max_retries
                    )));
                }
            }
        }
    }
}

fn should_retry(error: &CoreError) -> bool {
    match error {
        CoreError::Api(api_error) => api_error.is_retryable(),
        CoreError::Cache(_) => true,
        CoreError::Retry(_) => true,
        CoreError::Other(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiError;

    #[tokio::test]
    async fn test_retry_success_after_failure() {
        let mut attempts = 0;
        let result = with_retry(
            || async {
                attempts += 1;
                if attempts < 2 {
                    Err(CoreError::Retry("Test retry".to_string()))
                } else {
                    Ok("success")
                }
            },
            3,
            Duration::from_millis(1),
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let mut attempts = 0;
        let result = with_retry(
            || async {
                attempts += 1;
                Err(CoreError::Retry("Test retry".to_string()))
            },
            2,
            Duration::from_millis(1),
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts, 2);
    }
}
