//! Response caching for faster repeated queries.
//!
//! This module provides file-based caching of API responses with configurable TTLs
//! for different data types. Cache files are stored in `~/.cache/lin/` or the
//! platform-appropriate cache directory.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::LinError;
use crate::Result;

/// Default TTL for teams (1 hour) - teams change infrequently.
pub const TTL_TEAMS: Duration = Duration::from_secs(60 * 60);

/// Default TTL for users (1 hour) - users change infrequently.
pub const TTL_USERS: Duration = Duration::from_secs(60 * 60);

/// Default TTL for workflow states (1 hour) - workflow states rarely change.
pub const TTL_WORKFLOW_STATES: Duration = Duration::from_secs(60 * 60);

/// Default TTL for labels (30 minutes) - labels change occasionally.
pub const TTL_LABELS: Duration = Duration::from_secs(30 * 60);

/// Default TTL for projects (15 minutes) - projects change moderately.
pub const TTL_PROJECTS: Duration = Duration::from_secs(15 * 60);

/// Default TTL for cycles (15 minutes) - cycles change moderately.
pub const TTL_CYCLES: Duration = Duration::from_secs(15 * 60);

/// Default TTL for documents (10 minutes) - documents change more frequently.
pub const TTL_DOCUMENTS: Duration = Duration::from_secs(10 * 60);

/// Default TTL for issues (5 minutes) - issues change frequently.
pub const TTL_ISSUES: Duration = Duration::from_secs(5 * 60);

/// Default TTL for comments (5 minutes) - comments change frequently.
pub const TTL_COMMENTS: Duration = Duration::from_secs(5 * 60);

/// Default TTL for search results (2 minutes) - search results should be fresh.
pub const TTL_SEARCH: Duration = Duration::from_secs(2 * 60);

/// Cache entry wrapper that stores the data along with metadata.
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry<T> {
    /// The cached data.
    data: T,
    /// Unix timestamp when the entry was created.
    created_at: u64,
    /// TTL in seconds for this entry.
    ttl_secs: u64,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with the given data and TTL.
    fn new(data: T, ttl: Duration) -> Self {
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            data,
            created_at,
            ttl_secs: ttl.as_secs(),
        }
    }

    /// Check if this cache entry has expired.
    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now > self.created_at + self.ttl_secs
    }
}

/// File-based cache for API responses.
///
/// The cache stores responses as JSON files in the cache directory,
/// with filenames based on a hash of the request parameters.
#[derive(Debug, Clone)]
pub struct Cache {
    /// The base directory for cache files.
    cache_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new cache instance using the default cache directory.
    ///
    /// The cache directory is `~/.cache/lin/` on Unix systems or the
    /// platform-appropriate cache directory on other platforms.
    pub fn new() -> Self {
        let cache_dir = Self::default_cache_dir();
        Self { cache_dir }
    }

    /// Create a new cache instance with a custom cache directory.
    ///
    /// This is primarily useful for testing.
    pub fn with_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get the default cache directory path.
    pub fn default_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lin")
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Generate a cache key from the query and variables.
    ///
    /// The key is a hash of the query string and variables JSON,
    /// resulting in a consistent filename for the same request.
    pub fn generate_key(query: &str, variables: &serde_json::Value) -> String {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        variables.to_string().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get the file path for a cache entry.
    fn entry_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key))
    }

    /// Get a cached response if it exists and hasn't expired.
    ///
    /// Returns `None` if:
    /// - The cache entry doesn't exist
    /// - The cache entry has expired
    /// - The cache entry can't be read or parsed
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.entry_path(key);

        if !path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&path).ok()?;
        let entry: CacheEntry<T> = serde_json::from_str(&contents).ok()?;

        if entry.is_expired() {
            // Remove expired entry
            let _ = fs::remove_file(&path);
            return None;
        }

        Some(entry.data)
    }

    /// Store a response in the cache with the given TTL.
    ///
    /// Creates the cache directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cache directory can't be created
    /// - The cache entry can't be written
    pub fn set<T: Serialize>(&self, key: &str, data: &T, ttl: Duration) -> Result<()> {
        // Ensure cache directory exists
        fs::create_dir_all(&self.cache_dir)?;

        let entry = CacheEntry::new(data, ttl);
        let json = serde_json::to_string(&entry)
            .map_err(|e| LinError::parse(format!("Failed to serialize cache entry: {}", e)))?;

        let path = self.entry_path(key);
        fs::write(&path, json)?;

        Ok(())
    }

    /// Clear all cached entries.
    ///
    /// Removes all files in the cache directory.
    ///
    /// # Returns
    ///
    /// Returns the number of entries that were cleared.
    pub fn clear(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                fs::remove_file(&path)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get statistics about the cache.
    ///
    /// Returns information about the cache size, entry count, and expired entries.
    pub fn stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats::default();

        if !self.cache_dir.exists() {
            return Ok(stats);
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                stats.total_entries += 1;

                if let Ok(metadata) = fs::metadata(&path) {
                    stats.total_size_bytes += metadata.len();
                }

                // Check if entry is expired by reading it
                if let Ok(contents) = fs::read_to_string(&path) {
                    // Parse just the metadata portion to check expiry
                    if let Ok(entry) =
                        serde_json::from_str::<CacheEntry<serde_json::Value>>(&contents)
                    {
                        if entry.is_expired() {
                            stats.expired_entries += 1;
                        }
                    }
                }
            }
        }

        stats.valid_entries = stats.total_entries - stats.expired_entries;

        Ok(stats)
    }

    /// Remove all expired entries from the cache.
    ///
    /// # Returns
    ///
    /// Returns the number of expired entries that were removed.
    pub fn prune_expired(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(entry) =
                        serde_json::from_str::<CacheEntry<serde_json::Value>>(&contents)
                    {
                        if entry.is_expired() {
                            fs::remove_file(&path)?;
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }
}

/// Statistics about the cache.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheStats {
    /// Total number of cache entries (including expired).
    pub total_entries: usize,
    /// Number of valid (non-expired) entries.
    pub valid_entries: usize,
    /// Number of expired entries.
    pub expired_entries: usize,
    /// Total size of all cache files in bytes.
    pub total_size_bytes: u64,
}

impl CacheStats {
    /// Format the size in a human-readable format.
    pub fn formatted_size(&self) -> String {
        let bytes = self.total_size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::TempDir;

    fn create_test_cache() -> (Cache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = Cache::with_dir(temp_dir.path().to_path_buf());
        (cache, temp_dir)
    }

    #[test]
    fn test_cache_new() {
        let cache = Cache::new();
        assert!(cache.cache_dir().to_string_lossy().contains("lin"));
    }

    #[test]
    fn test_generate_key() {
        let key1 = Cache::generate_key("query { viewer { id } }", &serde_json::json!({}));
        let key2 = Cache::generate_key("query { viewer { id } }", &serde_json::json!({}));
        let key3 = Cache::generate_key("query { viewer { name } }", &serde_json::json!({}));

        // Same query + variables should produce the same key
        assert_eq!(key1, key2);
        // Different query should produce a different key
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_generate_key_with_variables() {
        let key1 = Cache::generate_key("query", &serde_json::json!({"id": "123"}));
        let key2 = Cache::generate_key("query", &serde_json::json!({"id": "456"}));

        // Different variables should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_set_and_get() {
        let (cache, _temp) = create_test_cache();

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let key = "test_key";
        cache.set(key, &data, Duration::from_secs(60)).unwrap();

        let retrieved: Option<TestData> = cache.get(key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let (cache, _temp) = create_test_cache();

        let retrieved: Option<String> = cache.get("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let (cache, _temp) = create_test_cache();

        let data = "test data";
        let key = "expiring_key";

        // Set with 1 second TTL (TTL is stored in seconds, so milliseconds don't work)
        cache.set(key, &data, Duration::from_secs(1)).unwrap();

        // Should be available immediately
        let retrieved: Option<String> = cache.get(key);
        assert!(retrieved.is_some());

        // Wait for expiration (sleep 2 seconds to ensure TTL has passed)
        thread::sleep(Duration::from_secs(2));

        // Should be gone now
        let retrieved: Option<String> = cache.get(key);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_cache_clear() {
        let (cache, _temp) = create_test_cache();

        // Add some entries
        cache
            .set("key1", &"value1", Duration::from_secs(60))
            .unwrap();
        cache
            .set("key2", &"value2", Duration::from_secs(60))
            .unwrap();
        cache
            .set("key3", &"value3", Duration::from_secs(60))
            .unwrap();

        // Verify they exist
        assert!(cache.get::<String>("key1").is_some());
        assert!(cache.get::<String>("key2").is_some());
        assert!(cache.get::<String>("key3").is_some());

        // Clear the cache
        let count = cache.clear().unwrap();
        assert_eq!(count, 3);

        // Verify they're gone
        assert!(cache.get::<String>("key1").is_none());
        assert!(cache.get::<String>("key2").is_none());
        assert!(cache.get::<String>("key3").is_none());
    }

    #[test]
    fn test_cache_clear_empty() {
        let (cache, _temp) = create_test_cache();

        let count = cache.clear().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_cache_stats() {
        let (cache, _temp) = create_test_cache();

        // Empty cache
        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.valid_entries, 0);
        assert_eq!(stats.expired_entries, 0);

        // Add some entries
        cache
            .set("key1", &"value1", Duration::from_secs(60))
            .unwrap();
        cache
            .set("key2", &"value2", Duration::from_secs(60))
            .unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 2);
        assert_eq!(stats.expired_entries, 0);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_cache_stats_with_expired() {
        let (cache, _temp) = create_test_cache();

        // Add a valid entry
        cache
            .set("valid", &"value", Duration::from_secs(60))
            .unwrap();

        // Add an expired entry using 1 second TTL and wait for it to expire
        cache
            .set("expired", &"value", Duration::from_secs(1))
            .unwrap();
        thread::sleep(Duration::from_secs(2));

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 1);
        assert_eq!(stats.expired_entries, 1);
    }

    #[test]
    fn test_cache_prune_expired() {
        let (cache, _temp) = create_test_cache();

        // Add a valid entry
        cache
            .set("valid", &"value", Duration::from_secs(60))
            .unwrap();

        // Add entries that will expire (1 second TTL)
        cache
            .set("expired1", &"value", Duration::from_secs(1))
            .unwrap();
        cache
            .set("expired2", &"value", Duration::from_secs(1))
            .unwrap();
        thread::sleep(Duration::from_secs(2));

        // Prune expired entries
        let count = cache.prune_expired().unwrap();
        assert_eq!(count, 2);

        // Verify only valid entry remains
        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.valid_entries, 1);
    }

    #[test]
    fn test_formatted_size() {
        let stats = CacheStats {
            total_entries: 0,
            valid_entries: 0,
            expired_entries: 0,
            total_size_bytes: 500,
        };
        assert_eq!(stats.formatted_size(), "500 B");

        let stats = CacheStats {
            total_entries: 0,
            valid_entries: 0,
            expired_entries: 0,
            total_size_bytes: 2048,
        };
        assert_eq!(stats.formatted_size(), "2.0 KB");

        let stats = CacheStats {
            total_entries: 0,
            valid_entries: 0,
            expired_entries: 0,
            total_size_bytes: 1024 * 1024 * 2,
        };
        assert_eq!(stats.formatted_size(), "2.0 MB");
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("test", Duration::from_secs(60));
        assert!(!entry.is_expired());

        let entry = CacheEntry {
            data: "test",
            created_at: 0, // Very old timestamp
            ttl_secs: 60,
        };
        assert!(entry.is_expired());
    }

    #[test]
    fn test_default_cache_dir() {
        let dir = Cache::default_cache_dir();
        assert!(dir.to_string_lossy().contains("lin"));
    }

    #[test]
    fn test_cache_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("subdir").join("cache");
        let cache = Cache::with_dir(cache_dir.clone());

        // Directory shouldn't exist yet
        assert!(!cache_dir.exists());

        // Setting a value should create the directory
        cache.set("key", &"value", Duration::from_secs(60)).unwrap();

        // Now it should exist
        assert!(cache_dir.exists());
    }

    #[test]
    fn test_cache_complex_data() {
        let (cache, _temp) = create_test_cache();

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct ComplexData {
            items: Vec<String>,
            nested: NestedData,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct NestedData {
            value: i32,
            flag: bool,
        }

        let data = ComplexData {
            items: vec!["a".to_string(), "b".to_string()],
            nested: NestedData {
                value: 123,
                flag: true,
            },
        };

        cache
            .set("complex", &data, Duration::from_secs(60))
            .unwrap();

        let retrieved: Option<ComplexData> = cache.get("complex");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }
}
