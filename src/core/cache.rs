use std::time::Duration;
use cached::{TimedCache, Cached};
use std::sync::Mutex;

/// Cache for storing query responses
pub struct QueryCache {
    cache: Mutex<TimedCache<String, String>>,
}

impl QueryCache {
    /// Create a new query cache with the specified size and TTL
    pub fn new(size: usize, ttl: Duration) -> Self {
        Self {
            cache: Mutex::new(TimedCache::with_lifespan_and_capacity(
                ttl.as_secs() as u64,
                size,
            )),
        }
    }

    /// Get a cached response for a query
    pub fn get(&self, query: &str) -> Option<String> {
        self.cache
            .lock()
            .expect("Failed to lock cache")
            .cache_get(&query.to_string())
            .cloned()
    }

    /// Insert a response into the cache
    pub fn insert(&self, query: String, response: String) {
        self.cache
            .lock()
            .expect("Failed to lock cache")
            .cache_set(query, response);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache
            .lock()
            .expect("Failed to lock cache")
            .cache_clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.cache
            .lock()
            .expect("Failed to lock cache")
            .cache_size()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let cache = QueryCache::new(10, Duration::from_secs(60));

        // Test insert and get
        cache.insert("test query".to_string(), "test response".to_string());
        assert_eq!(
            cache.get("test query"),
            Some("test response".to_string())
        );

        // Test size
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        // Test clear
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.get("test query"), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = QueryCache::new(10, Duration::from_millis(100));

        cache.insert("test query".to_string(), "test response".to_string());
        assert_eq!(
            cache.get("test query"),
            Some("test response".to_string())
        );

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(cache.get("test query"), None);
    }

    #[test]
    fn test_cache_capacity() {
        let cache = QueryCache::new(2, Duration::from_secs(60));

        cache.insert("query1".to_string(), "response1".to_string());
        cache.insert("query2".to_string(), "response2".to_string());
        cache.insert("query3".to_string(), "response3".to_string());

        // The oldest entry should be evicted
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get("query1"), None);
        assert_eq!(cache.get("query2"), Some("response2".to_string()));
        assert_eq!(cache.get("query3"), Some("response3".to_string()));
    }
}
