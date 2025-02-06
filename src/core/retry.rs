use std::time::Duration;
use backoff::{ExponentialBackoff, backoff::Backoff};
use futures::Future;
use tokio::time::sleep;

use super::{CoreError, CoreResult};

/// Execute an async operation with exponential backoff retry
pub async fn with_backoff<F, Fut, T>(
    operation: F,
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> CoreResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = CoreResult<T>>,
{
    let mut backoff = ExponentialBackoff {
        initial_interval: initial_delay,
        max_interval: max_delay,
        multiplier: 2.0,
        max_elapsed_time: Some(max_delay * max_retries),
        ..Default::default()
    };

    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(CoreError::Retry(format!(
                        "Operation failed after {} attempts: {}",
                        attempt, err
                    )));
                }

                if let Some(duration) = backoff.next_backoff() {
                    sleep(duration).await;
                } else {
                    return Err(CoreError::Retry(
                        "Retry timeout exceeded".to_string(),
                    ));
                }
            }
        }
    }
}

/// Determine if an error should be retried
pub fn should_retry(error: &CoreError) -> bool {
    match error {
        CoreError::Api(api_error) => match api_error {
            crate::api::ApiError::Network(_) => true,
            crate::api::ApiError::RateLimit => true,
            _ => false,
        },
        CoreError::Stream(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let operation = move || {
            let current_attempt = attempts_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if current_attempt < 2 {
                    Err(CoreError::Retry("test error".to_string()))
                } else {
                    Ok("success")
                }
            }
        };

        let result = with_backoff(
            operation,
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let operation = || async {
            Err(CoreError::Retry("test error".to_string()))
        };

        let result = with_backoff(
            operation,
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CoreError::Retry(_))));
    }

    #[tokio::test]
    async fn test_should_retry() {
        assert!(should_retry(&CoreError::Api(crate::api::ApiError::Network(
            reqwest::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "network error"
            ))
        ))));
        assert!(should_retry(&CoreError::Api(crate::api::ApiError::RateLimit)));
        assert!(!should_retry(&CoreError::Api(crate::api::ApiError::InvalidKey)));
        assert!(should_retry(&CoreError::Stream("stream error".to_string())));
        assert!(!should_retry(&CoreError::Cache("cache error".to_string())));
    }
}
