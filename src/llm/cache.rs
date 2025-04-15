use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use crate::monitoring;

use crate::llm::client::{LlmRequest, LlmResponse};

/// Cache entry for LLM responses
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Response data
    response: LlmResponse,

    /// Timestamp when the entry was created
    created_at: u64,

    /// Timestamp when the entry expires
    expires_at: u64,
}

/// Cache metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheMetrics {
    /// Number of cache hits
    pub hits: u64,

    /// Number of cache misses
    pub misses: u64,

    /// Number of cache entries
    pub entries: u64,

    /// Number of expired entries removed
    pub expired_removed: u64,

    /// Total size of cached responses in bytes
    pub total_size_bytes: u64,

    /// Creation time of the cache
    pub created_at: u64,

    /// Last access time
    pub last_access: u64,
}

/// LLM response cache
#[derive(Debug)]
pub struct ResponseCache {
    /// Cache directory
    cache_dir: PathBuf,

    /// In-memory cache
    memory_cache: HashMap<String, CacheEntry>,

    /// Cache TTL in seconds
    ttl: u64,

    /// Whether to use disk cache
    use_disk: bool,

    /// Cache metrics
    metrics: CacheMetrics,
}

impl ResponseCache {
    /// Create a new response cache
    pub fn new(ttl_seconds: u64, use_disk: bool) -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;

        // Create the cache directory if it doesn't exist
        if use_disk && !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        // Initialize metrics
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut metrics = CacheMetrics {
            created_at: now,
            last_access: now,
            ..Default::default()
        };

        // Count existing entries if using disk cache
        if use_disk && cache_dir.exists() {
            if let Ok(entries) = fs::read_dir(&cache_dir) {
                let mut entry_count = 0;
                let mut total_size = 0;

                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            entry_count += 1;
                            total_size += metadata.len();
                        }
                    }
                }

                metrics.entries = entry_count;
                metrics.total_size_bytes = total_size;
            }
        }

        Ok(Self {
            cache_dir,
            memory_cache: HashMap::new(),
            ttl: ttl_seconds,
            use_disk,
            metrics,
        })
    }

    /// Get the cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
            .join("qitops")
            .join("llm_cache");

        Ok(cache_dir)
    }

    /// Generate a cache key for a request
    fn generate_key(&self, request: &LlmRequest, provider: &str) -> String {
        // Create a simple hash of the request and provider
        // In a real implementation, we would use a more sophisticated hashing algorithm
        let mut key = format!("{}-{}", provider, request.model);

        // Add messages to the key
        for message in &request.messages {
            key.push_str(&format!("-{}-{}", message.role, message.content));
        }

        // Hash the key
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get the path to a cache file
    fn get_cache_file(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key))
    }

    /// Get a response from the cache
    pub fn get(&mut self, request: &LlmRequest, provider: &str) -> Option<LlmResponse> {
        let key = self.generate_key(request, provider);

        // Update last access time
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.metrics.last_access = now;

        // Check memory cache first
        if let Some(entry) = self.memory_cache.get(&key) {
            if entry.expires_at > now {
                // Cache hit in memory
                self.metrics.hits += 1;
                // Also track in monitoring
                monitoring::track_cache_hit();
                return Some(entry.response.clone());
            }
        }

        // If not in memory cache and disk cache is enabled, check disk
        if self.use_disk {
            let cache_file = self.get_cache_file(&key);
            if cache_file.exists() {
                if let Ok(content) = fs::read_to_string(&cache_file) {
                    if let Ok(entry) = serde_json::from_str::<CacheEntry>(&content) {
                        if entry.expires_at > now {
                            // Cache hit on disk
                            self.metrics.hits += 1;
                            // Also track in monitoring
                            monitoring::track_cache_hit();

                            // Add to memory cache for faster access next time
                            self.memory_cache.insert(key.clone(), entry.clone());

                            return Some(entry.response.clone());
                        } else {
                            // Entry is expired, remove it
                            if fs::remove_file(&cache_file).is_ok() {
                                self.metrics.expired_removed += 1;

                                // Update total size
                                if let Ok(metadata) = fs::metadata(&cache_file) {
                                    self.metrics.total_size_bytes = self.metrics.total_size_bytes.saturating_sub(metadata.len());
                                }

                                // Update entry count
                                self.metrics.entries = self.metrics.entries.saturating_sub(1);
                            }
                        }
                    }
                }
            }
        }

        // Cache miss
        self.metrics.misses += 1;
        // Also track in monitoring
        monitoring::track_cache_miss();
        None
    }

    /// Put a response in the cache
    pub fn put(&mut self, request: &LlmRequest, provider: &str, response: LlmResponse) -> Result<()> {
        let key = self.generate_key(request, provider);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = CacheEntry {
            response: response.clone(),
            created_at: now,
            expires_at: now + self.ttl,
        };

        // Update last access time
        self.metrics.last_access = now;

        // Check if this is a new entry
        let is_new_entry = !self.memory_cache.contains_key(&key);

        // Add to memory cache
        self.memory_cache.insert(key.clone(), entry.clone());

        // If disk cache is enabled, write to disk
        if self.use_disk {
            let cache_file = self.get_cache_file(&key);
            let content = serde_json::to_string(&entry)?;

            // Calculate size difference for metrics
            let old_size = if cache_file.exists() {
                fs::metadata(&cache_file).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            // Write to disk
            fs::write(&cache_file, &content)?;

            // Update metrics
            let new_size = content.len() as u64;

            if is_new_entry {
                // New entry
                self.metrics.entries += 1;
                self.metrics.total_size_bytes += new_size;
            } else {
                // Updated entry
                self.metrics.total_size_bytes = self.metrics.total_size_bytes.saturating_sub(old_size);
                self.metrics.total_size_bytes += new_size;
            }
        }

        Ok(())
    }

    /// Clear the cache
    pub fn clear(&mut self) -> Result<()> {
        // Clear memory cache
        self.memory_cache.clear();

        // If disk cache is enabled, clear disk cache
        if self.use_disk && self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    fs::remove_file(path)?;
                }
            }
        }

        // Reset metrics
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.metrics = CacheMetrics::default();
        self.metrics.created_at = now;
        self.metrics.last_access = now;

        Ok(())
    }

    /// Clean expired entries
    pub fn clean_expired(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Update last access time
        self.metrics.last_access = now;

        // Count expired entries in memory cache
        let _memory_expired_count = self.memory_cache.iter()
            .filter(|(_, entry)| entry.expires_at <= now)
            .count() as u64;

        // Clean memory cache
        self.memory_cache.retain(|_, entry| entry.expires_at > now);

        // If disk cache is enabled, clean disk cache
        if self.use_disk && self.cache_dir.exists() {
            let mut disk_expired_count = 0;
            let mut size_removed = 0;

            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(entry) = serde_json::from_str::<CacheEntry>(&content) {
                            if entry.expires_at <= now {
                                // Get file size before removing
                                if let Ok(metadata) = fs::metadata(&path) {
                                    size_removed += metadata.len();
                                }

                                // Remove the file
                                if fs::remove_file(&path).is_ok() {
                                    disk_expired_count += 1;
                                }
                            }
                        }
                    }
                }
            }

            // Update metrics
            self.metrics.expired_removed += disk_expired_count;
            self.metrics.entries = self.metrics.entries.saturating_sub(disk_expired_count);
            self.metrics.total_size_bytes = self.metrics.total_size_bytes.saturating_sub(size_removed);
        }

        Ok(())
    }

    /// Get cache metrics
    pub fn get_metrics(&self) -> &CacheMetrics {
        &self.metrics
    }
}
