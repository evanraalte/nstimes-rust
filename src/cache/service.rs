use super::models::CacheEntry;
use std::collections::HashMap;
use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Mutex;

/// Price cache that stores prices with expiration dates
pub struct PriceCache {
    /// Path to the cache file
    path: String,
    /// In-memory cache entries (uses interior mutability for thread-safe updates)
    /// Key format: "station1-station2-class" where stations are alphabetically sorted
    entries: Mutex<HashMap<String, CacheEntry>>,
}

impl PriceCache {
    /// Load or create a new price cache from the given file path
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let entries = if path.as_ref().exists() {
            // Load existing cache
            let content = fs::read_to_string(&path)?;
            match serde_json::from_str(&content) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("⚠️  Failed to parse cache file, starting fresh: {}", e);
                    HashMap::new()
                }
            }
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.as_ref().parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            HashMap::new()
        };

        Ok(Self {
            path: path_str,
            entries: Mutex::new(entries),
        })
    }

    /// Get a cached price for a station pair and travel class
    /// Returns None if not found or expired
    pub fn get(
        &self,
        from: &str,
        to: &str,
        travel_class: u8,
    ) -> Option<u32> {
        let key = Self::normalize_key(from, to, travel_class);

        let entries = self.entries.lock().ok()?;
        if let Some(entry) = entries.get(&key) {
            if !entry.is_expired() {
                return Some(entry.price_cents);
            }
        }

        None
    }

    /// Set a cached price for a station pair and travel class
    /// Automatically calculates expiration date (next January 1st)
    pub fn set(
        &self,
        from: &str,
        to: &str,
        travel_class: u8,
        price_cents: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = Self::normalize_key(from, to, travel_class);
        let entry = CacheEntry::new(price_cents, travel_class);

        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key, entry);
            drop(entries); // Release lock before saving
            self.save()?;
        }

        Ok(())
    }

    /// Normalize station pair into a consistent cache key
    /// A->B is the same as B->A, so we sort alphabetically
    /// Format: "station1-station2-class"
    fn normalize_key(from: &str, to: &str, travel_class: u8) -> String {
        let (first, second) = if from < to {
            (from, to)
        } else {
            (to, from)
        };

        format!("{}-{}-{}", first, second, travel_class)
    }

    /// Save the cache to disk
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.entries.lock().map_err(|_| "Failed to lock cache")?;
        let file = fs::File::create(&self.path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &*entries)?;
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        if let Ok(entries) = self.entries.lock() {
            let total = entries.len();
            let expired = entries.values().filter(|e| e.is_expired()).count();
            let valid = total - expired;

            CacheStats {
                total_entries: total,
                valid_entries: valid,
                expired_entries: expired,
            }
        } else {
            CacheStats {
                total_entries: 0,
                valid_entries: 0,
                expired_entries: 0,
            }
        }
    }

    /// Clean up expired entries from the cache
    pub fn cleanup(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let removed = if let Ok(mut entries) = self.entries.lock() {
            let before = entries.len();
            entries.retain(|_, entry| !entry.is_expired());
            before - entries.len()
        } else {
            0
        };

        if removed > 0 {
            self.save()?;
        }

        Ok(removed)
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_normalize_key() {
        // A->B should equal B->A
        let key1 = PriceCache::normalize_key("Amsterdam", "Utrecht", 2);
        let key2 = PriceCache::normalize_key("Utrecht", "Amsterdam", 2);
        assert_eq!(key1, key2);
        assert_eq!(key1, "Amsterdam-Utrecht-2");

        // Different class should produce different key
        let key3 = PriceCache::normalize_key("Amsterdam", "Utrecht", 1);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_operations() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = env::temp_dir();
        let cache_path = temp_dir.join("test_cache.json");

        // Clean up any existing test cache
        let _ = fs::remove_file(&cache_path);

        // Create new cache
        let cache = PriceCache::new(&cache_path)?;

        // Set a price
        cache.set("Amsterdam", "Utrecht", 2, 940)?;

        // Get it back
        let price = cache.get("Amsterdam", "Utrecht", 2);
        assert_eq!(price, Some(940));

        // Check reverse direction works
        let price_reverse = cache.get("Utrecht", "Amsterdam", 2);
        assert_eq!(price_reverse, Some(940));

        // Different class should not match
        let price_different_class = cache.get("Amsterdam", "Utrecht", 1);
        assert_eq!(price_different_class, None);

        // Load cache from disk
        let cache2 = PriceCache::new(&cache_path)?;
        let price_reloaded = cache2.get("Amsterdam", "Utrecht", 2);
        assert_eq!(price_reloaded, Some(940));

        // Clean up
        fs::remove_file(&cache_path)?;

        Ok(())
    }
}
