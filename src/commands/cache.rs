//! Cache management commands.
//!
//! Commands for viewing and managing the API response cache.

use serde::Serialize;

use crate::Result;
use crate::cache::{Cache, CacheStats};
use crate::output::{HumanDisplay, OutputFormat, output};

/// Response for cache clear command.
#[derive(Debug, Serialize)]
pub struct CacheClearResponse {
    /// Number of entries that were cleared.
    pub entries_cleared: usize,
    /// Message describing the action.
    pub message: String,
}

impl HumanDisplay for CacheClearResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

/// Response for cache status command.
#[derive(Debug, Serialize)]
pub struct CacheStatusResponse {
    /// Cache directory path.
    pub cache_dir: String,
    /// Total number of cache entries.
    pub total_entries: usize,
    /// Number of valid (non-expired) entries.
    pub valid_entries: usize,
    /// Number of expired entries.
    pub expired_entries: usize,
    /// Total size in bytes.
    pub total_size_bytes: u64,
    /// Human-readable size.
    pub formatted_size: String,
}

impl From<CacheStats> for CacheStatusResponse {
    fn from(stats: CacheStats) -> Self {
        Self {
            cache_dir: Cache::default_cache_dir().to_string_lossy().to_string(),
            total_entries: stats.total_entries,
            valid_entries: stats.valid_entries,
            expired_entries: stats.expired_entries,
            total_size_bytes: stats.total_size_bytes,
            formatted_size: stats.formatted_size(),
        }
    }
}

impl HumanDisplay for CacheStatusResponse {
    fn human_fmt(&self) -> String {
        use colored::Colorize;

        let mut parts = vec![format!("{}", "Cache Status".bold())];
        parts.push(format!("  {}: {}", "Location".dimmed(), self.cache_dir));
        parts.push(format!(
            "  {}: {} ({} valid, {} expired)",
            "Entries".dimmed(),
            self.total_entries,
            self.valid_entries.to_string().green(),
            if self.expired_entries > 0 {
                self.expired_entries.to_string().yellow().to_string()
            } else {
                self.expired_entries.to_string()
            }
        ));
        parts.push(format!("  {}: {}", "Size".dimmed(), self.formatted_size));

        parts.join("\n")
    }
}

/// Clear all cached entries.
///
/// Removes all cache files from the cache directory.
pub fn clear_cache(format: OutputFormat) -> Result<()> {
    let cache = Cache::new();
    let count = cache.clear()?;

    let response = CacheClearResponse {
        entries_cleared: count,
        message: if count == 0 {
            "Cache is already empty.".to_string()
        } else if count == 1 {
            "Cleared 1 cache entry.".to_string()
        } else {
            format!("Cleared {} cache entries.", count)
        },
    };

    output(&response, format);
    Ok(())
}

/// Show cache statistics.
///
/// Displays information about the cache including size, entry count, and expired entries.
pub fn cache_status(format: OutputFormat) -> Result<()> {
    let cache = Cache::new();
    let stats = cache.stats()?;
    let response = CacheStatusResponse::from(stats);

    output(&response, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_clear_response_human_fmt() {
        let response = CacheClearResponse {
            entries_cleared: 5,
            message: "Cleared 5 cache entries.".to_string(),
        };
        assert_eq!(response.human_fmt(), "Cleared 5 cache entries.");
    }

    #[test]
    fn test_cache_status_response_from_stats() {
        let stats = CacheStats {
            total_entries: 10,
            valid_entries: 8,
            expired_entries: 2,
            total_size_bytes: 2048,
        };

        let response = CacheStatusResponse::from(stats);
        assert_eq!(response.total_entries, 10);
        assert_eq!(response.valid_entries, 8);
        assert_eq!(response.expired_entries, 2);
        assert_eq!(response.total_size_bytes, 2048);
        assert_eq!(response.formatted_size, "2.0 KB");
    }

    #[test]
    fn test_cache_status_response_human_fmt() {
        let response = CacheStatusResponse {
            cache_dir: "/home/user/.cache/lin".to_string(),
            total_entries: 10,
            valid_entries: 8,
            expired_entries: 2,
            total_size_bytes: 2048,
            formatted_size: "2.0 KB".to_string(),
        };

        let output = response.human_fmt();
        assert!(output.contains("Cache Status"));
        assert!(output.contains("/home/user/.cache/lin"));
        assert!(output.contains("10"));
        assert!(output.contains("8"));
        assert!(output.contains("2"));
        assert!(output.contains("2.0 KB"));
    }
}
