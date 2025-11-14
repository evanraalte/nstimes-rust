use serde::{Deserialize, Serialize};

/// A cached price entry with expiration date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Price in cents
    pub price_cents: u32,
    /// Travel class (1 or 2)
    pub travel_class: u8,
    /// Expiration date in ISO format (YYYY-MM-DD)
    /// Prices expire on January 1st each year
    pub expires_at: String,
}

impl CacheEntry {
    /// Create a new cache entry with expiration set to next January 1st
    pub fn new(price_cents: u32, travel_class: u8) -> Self {
        let expires_at = Self::next_january_first();
        Self {
            price_cents,
            travel_class,
            expires_at,
        }
    }

    /// Check if this cache entry has expired
    pub fn is_expired(&self) -> bool {
        use chrono::{Local, NaiveDate};

        let now = Local::now().date_naive();

        // Parse the expiration date
        if let Ok(expiry_date) = NaiveDate::parse_from_str(&self.expires_at, "%Y-%m-%d") {
            now >= expiry_date
        } else {
            // If we can't parse the date, consider it expired
            true
        }
    }

    /// Calculate the next January 1st from today
    fn next_january_first() -> String {
        use chrono::{Datelike, Local};

        let now = Local::now();
        let current_year = now.year();

        // Next January 1st is always in the next year
        let next_year = current_year + 1;
        format!("{}-01-01", next_year)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiration_format() {
        let entry = CacheEntry::new(1000, 2);
        assert!(entry.expires_at.starts_with("20")); // Year starts with 20
        assert!(entry.expires_at.ends_with("-01-01")); // Ends with Jan 1st
    }

    #[test]
    fn test_is_expired() {
        // Entry that expired in 2020
        let expired = CacheEntry {
            price_cents: 1000,
            travel_class: 2,
            expires_at: "2020-01-01".to_string(),
        };
        assert!(expired.is_expired());

        // Entry that expires far in the future
        let valid = CacheEntry {
            price_cents: 1000,
            travel_class: 2,
            expires_at: "2099-01-01".to_string(),
        };
        assert!(!valid.is_expired());
    }
}
