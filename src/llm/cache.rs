use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

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

/// LLM response cache
pub struct ResponseCache {
    /// Cache directory
    cache_dir: PathBuf,
    
    /// In-memory cache
    memory_cache: HashMap<String, CacheEntry>,
    
    /// Cache TTL in seconds
    ttl: u64,
    
    /// Whether to use disk cache
    use_disk: bool,
}

impl ResponseCache {
    /// Create a new response cache
    pub fn new(ttl_seconds: u64, use_disk: bool) -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        
        // Create the cache directory if it doesn't exist
        if use_disk && !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        
        Ok(Self {
            cache_dir,
            memory_cache: HashMap::new(),
            ttl: ttl_seconds,
            use_disk,
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
    pub fn get(&self, request: &LlmRequest, provider: &str) -> Option<LlmResponse> {
        let key = self.generate_key(request, provider);
        
        // Check memory cache first
        if let Some(entry) = self.memory_cache.get(&key) {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
                
            if entry.expires_at > now {
                return Some(entry.response.clone());
            }
        }
        
        // If not in memory cache and disk cache is enabled, check disk
        if self.use_disk {
            let cache_file = self.get_cache_file(&key);
            if cache_file.exists() {
                if let Ok(content) = fs::read_to_string(&cache_file) {
                    if let Ok(entry) = serde_json::from_str::<CacheEntry>(&content) {
                        let now = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                            
                        if entry.expires_at > now {
                            return Some(entry.response.clone());
                        } else {
                            // Entry is expired, remove it
                            let _ = fs::remove_file(&cache_file);
                        }
                    }
                }
            }
        }
        
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
        
        // Add to memory cache
        self.memory_cache.insert(key.clone(), entry.clone());
        
        // If disk cache is enabled, write to disk
        if self.use_disk {
            let cache_file = self.get_cache_file(&key);
            let content = serde_json::to_string(&entry)?;
            fs::write(cache_file, content)?;
        }
        
        Ok(())
    }
    
    /// Clear the cache
    pub fn clear(&mut self) -> Result<()> {
        // Clear memory cache
        self.memory_cache.clear();
        
        // If disk cache is enabled, clear disk cache
        if self.use_disk {
            if self.cache_dir.exists() {
                for entry in fs::read_dir(&self.cache_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                        fs::remove_file(path)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Clean expired entries
    pub fn clean_expired(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Clean memory cache
        self.memory_cache.retain(|_, entry| entry.expires_at > now);
        
        // If disk cache is enabled, clean disk cache
        if self.use_disk {
            if self.cache_dir.exists() {
                for entry in fs::read_dir(&self.cache_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(entry) = serde_json::from_str::<CacheEntry>(&content) {
                                if entry.expires_at <= now {
                                    fs::remove_file(path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}
