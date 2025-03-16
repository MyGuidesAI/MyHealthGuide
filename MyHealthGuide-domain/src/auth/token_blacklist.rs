use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tracing::{debug, info, warn};
use once_cell::sync::Lazy;

/// Global token blacklist for revoked tokens
///
/// This static instance provides a singleton access point to the token blacklist
/// throughout the application. It's thread-safe and can be accessed from multiple
/// threads concurrently.
///
/// # Example
/// ```rust
/// use MyHealthGuide_domain::auth::token_blacklist;
///
/// // Check if a token is revoked
/// if token_blacklist::blacklist().is_revoked("some-token-id") {
///     println!("Token is revoked");
/// }
/// ```
static TOKEN_BLACKLIST: Lazy<TokenBlacklist> = Lazy::new(|| {
    TokenBlacklist::new()
});

/// Token blacklist structure for tracking revoked tokens
///
/// This structure maintains a thread-safe collection of revoked tokens
/// and provides methods to:
/// - Revoke tokens
/// - Check if a token is revoked
/// - Clean up expired tokens
///
/// The blacklist has a maximum size limit to prevent unbounded growth, and
/// it automatically removes expired tokens during cleanup operations.
pub struct TokenBlacklist {
    /// Map of token identifiers to expiration times
    /// Key: user_id or jti (JWT ID) if available
    /// Value: (expiration timestamp, revocation timestamp)
    revoked_tokens: Arc<Mutex<HashMap<String, (SystemTime, SystemTime)>>>,

    /// Maximum size of the blacklist before aggressive pruning
    max_size: usize,
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenBlacklist {
    /// Create a new token blacklist with default settings
    ///
    /// The default maximum size is 10,000 tokens.
    ///
    /// # Example
    /// ```rust
    /// use MyHealthGuide_domain::auth::token_blacklist::TokenBlacklist;
    ///
    /// let blacklist = TokenBlacklist::new();
    /// ```
    pub fn new() -> Self {
        Self {
            revoked_tokens: Arc::new(Mutex::new(HashMap::new())),
            max_size: 10000, // Default size limit
        }
    }

    /// Create a new token blacklist with custom maximum size
    ///
    /// This allows configuring the maximum number of tokens that can be stored
    /// in the blacklist before aggressive cleanup occurs.
    ///
    /// # Arguments
    /// * `max_size` - The maximum number of tokens to store
    ///
    /// # Example
    /// ```rust
    /// use MyHealthGuide_domain::auth::token_blacklist::TokenBlacklist;
    ///
    /// // Create a blacklist that can store up to 5000 tokens
    /// let blacklist = TokenBlacklist::with_max_size(5000);
    /// ```
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            revoked_tokens: Arc::new(Mutex::new(HashMap::new())),
            max_size,
        }
    }

    /// Add a token to the blacklist with specific expiration
    ///
    /// When the blacklist reaches its maximum size, it will first attempt to
    /// remove expired tokens. If still at capacity, it will remove the oldest
    /// tokens based on revocation time.
    ///
    /// # Arguments
    /// * `token_id` - A unique identifier for the token (usually JTI or user ID)
    /// * `expiration` - When the token expires naturally
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use MyHealthGuide_domain::auth::token_blacklist::TokenBlacklist;
    ///
    /// let blacklist = TokenBlacklist::new();
    /// let expiration = SystemTime::now() + Duration::from_secs(3600); // 1 hour expiration
    /// blacklist.revoke_token("user123:session456", expiration);
    /// ```
    pub fn revoke_token(&self, token_id: &str, expiration: SystemTime) {
        let revocation_time = SystemTime::now();
        let mut tokens = self.revoked_tokens.lock().unwrap();

        // Check size before adding
        if tokens.len() >= self.max_size {
            warn!("Token blacklist reached max size ({}), performing aggressive cleanup", self.max_size);
            self.cleanup_expired_tokens_internal(&mut tokens);

            // If still at capacity, remove oldest entries
            if tokens.len() >= self.max_size {
                self.remove_oldest_entries(&mut tokens, self.max_size / 2);
            }
        }

        // Add the token to the blacklist
        tokens.insert(token_id.to_string(), (expiration, revocation_time));
        info!("Token revoked: {}", token_id);
    }

    /// Check if a token is in the blacklist (has been revoked)
    ///
    /// # Arguments
    /// * `token_id` - The unique identifier for the token to check
    ///
    /// # Returns
    /// `true` if the token has been revoked, `false` otherwise
    ///
    /// # Example
    /// ```rust
    /// use MyHealthGuide_domain::auth::token_blacklist::TokenBlacklist;
    ///
    /// let blacklist = TokenBlacklist::new();
    /// // ... revoke some tokens ...
    ///
    /// if blacklist.is_revoked("user123:session456") {
    ///     println!("This token has been revoked");
    /// } else {
    ///     println!("This token is still valid");
    /// }
    /// ```
    pub fn is_revoked(&self, token_id: &str) -> bool {
        let tokens = self.revoked_tokens.lock().unwrap();
        tokens.contains_key(token_id)
    }

    /// Get the number of tokens in the blacklist
    ///
    /// # Returns
    /// The current number of revoked tokens in the blacklist
    ///
    /// # Example
    /// ```rust
    /// use MyHealthGuide_domain::auth::token_blacklist::TokenBlacklist;
    ///
    /// let blacklist = TokenBlacklist::new();
    /// println!("Blacklist contains {} revoked tokens", blacklist.size());
    /// ```
    pub fn size(&self) -> usize {
        let tokens = self.revoked_tokens.lock().unwrap();
        tokens.len()
    }

    /// Remove expired tokens from the blacklist
    ///
    /// This method should be called periodically to clean up the blacklist
    /// and prevent it from growing too large.
    ///
    /// # Returns
    /// The number of tokens that were removed
    ///
    /// # Example
    /// ```rust
    /// use MyHealthGuide_domain::auth::token_blacklist::TokenBlacklist;
    ///
    /// let blacklist = TokenBlacklist::new();
    /// // ... revoke some tokens ...
    ///
    /// let removed = blacklist.cleanup_expired_tokens();
    /// println!("Removed {} expired tokens", removed);
    /// ```
    pub fn cleanup_expired_tokens(&self) -> usize {
        let mut tokens = self.revoked_tokens.lock().unwrap();
        self.cleanup_expired_tokens_internal(&mut tokens)
    }

    /// Internal implementation of cleanup that works with an already-locked HashMap
    ///
    /// This method is used internally to avoid locking the HashMap multiple times
    /// when we already have a mutable reference to it.
    fn cleanup_expired_tokens_internal(&self, tokens: &mut HashMap<String, (SystemTime, SystemTime)>) -> usize {
        let now = SystemTime::now();
        let before_count = tokens.len();

        // Remove entries where the expiration time is in the past
        tokens.retain(|_, (expiration, _)| {
            now.duration_since(*expiration).is_err()
        });

        let removed = before_count - tokens.len();
        if removed > 0 {
            debug!("Removed {} expired tokens from blacklist", removed);
        }

        removed
    }

    /// Remove the oldest entries from the blacklist
    ///
    /// This is used as a fallback when cleanup_expired_tokens doesn't free up enough space.
    /// It sorts tokens by their revocation time and removes the oldest ones.
    fn remove_oldest_entries(&self, tokens: &mut HashMap<String, (SystemTime, SystemTime)>, count: usize) {
        // Clone the tokens to avoid borrow issues
        let entries_clone: Vec<(String, (SystemTime, SystemTime))> = tokens
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Sort by revocation time (oldest first)
        let mut sorted_entries = entries_clone.clone();
        sorted_entries.sort_by(|a, b| a.1.1.cmp(&b.1.1));

        // Take the oldest entries to remove (up to count)
        let to_remove: Vec<String> = sorted_entries.iter()
            .take(count)
            .map(|(k, _)| k.clone())
            .collect();

        // Remove these entries
        for key in to_remove {
            tokens.remove(key.as_str());
        }

        debug!("Removed {} oldest entries from token blacklist", count);
    }
}

/// Get a reference to the global token blacklist
///
/// This is the main entry point for interacting with the token blacklist.
///
/// # Returns
/// A reference to the singleton TokenBlacklist instance
///
/// # Example
/// ```rust
/// use MyHealthGuide_domain::auth::token_blacklist;
/// use std::time::{SystemTime, Duration};
///
/// // Revoke a token
/// let expiration = SystemTime::now() + Duration::from_secs(3600);
/// token_blacklist::blacklist().revoke_token("user123:session456", expiration);
///
/// // Check if a token is revoked
/// if token_blacklist::blacklist().is_revoked("user123:session456") {
///     println!("Token is revoked");
/// }
/// ```
pub fn blacklist() -> &'static TokenBlacklist {
    &TOKEN_BLACKLIST
}

/// Start a background task to periodically clean up the token blacklist
///
/// This function starts a Tokio task that runs every hour to remove expired tokens
/// from the blacklist. It should be called during application startup.
///
/// # Example
/// ```rust
/// // In your application startup code:
/// #[tokio::main]
/// async fn main() {
///     // ... other initialization ...
///     MyHealthGuide_domain::auth::token_blacklist::start_cleanup_task();
///     // ... continue with startup ...
/// }
/// ```
#[cfg(feature = "with-tokio")]
pub fn start_cleanup_task() {
    use tokio::time;
    use std::time::Duration;

    tokio::spawn(async move {
        let cleanup_interval = Duration::from_secs(3600); // 1 hour
        let mut interval = time::interval(cleanup_interval);

        loop {
            interval.tick().await;
            debug!("Running scheduled token blacklist cleanup");
            let removed = blacklist().cleanup_expired_tokens();
            debug!("Removed {} expired tokens, {} remain in blacklist",
                  removed, blacklist().size());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_revoke_and_check_token() {
        let blacklist = TokenBlacklist::new();

        // Set expiration to 1 second in the future
        let expiration = SystemTime::now() + Duration::from_secs(1);

        blacklist.revoke_token("test-token-1", expiration);

        // Token should be revoked
        assert!(blacklist.is_revoked("test-token-1"));

        // Unknown token should not be revoked
        assert!(!blacklist.is_revoked("unknown-token"));
    }

    #[test]
    fn test_cleanup_expired_tokens() {
        let blacklist = TokenBlacklist::new();

        // Add some tokens with different expirations
        let expired = SystemTime::now() - Duration::from_secs(1);
        let not_expired = SystemTime::now() + Duration::from_secs(60);

        blacklist.revoke_token("expired-token", expired);
        blacklist.revoke_token("valid-token", not_expired);

        // Verify both tokens are in the blacklist initially
        assert_eq!(blacklist.size(), 2);

        // Run cleanup
        let removed = blacklist.cleanup_expired_tokens();

        // One token should be removed
        assert_eq!(removed, 1);
        assert_eq!(blacklist.size(), 1);

        // The expired token should be gone
        assert!(!blacklist.is_revoked("expired-token"));

        // The valid token should still be there
        assert!(blacklist.is_revoked("valid-token"));
    }

    #[test]
    fn test_max_size_and_oldest_removal() {
        // Create a small blacklist for testing
        let blacklist = TokenBlacklist::with_max_size(5);

        // Add tokens up to max size
        for i in 0..5 {
            let expiration = SystemTime::now() + Duration::from_secs(300);
            blacklist.revoke_token(&format!("token-{}", i), expiration);
            // Small sleep to ensure different revocation times
            sleep(Duration::from_millis(10));
        }

        // Verify we have 5 tokens
        assert_eq!(blacklist.size(), 5);

        // Add another token, which should trigger cleanup of oldest
        let expiration = SystemTime::now() + Duration::from_secs(300);
        blacklist.revoke_token("new-token", expiration);

        // We should still have max size tokens
        assert_eq!(blacklist.size(), 5);

        // The oldest token should be gone
        assert!(!blacklist.is_revoked("token-0"));

        // The new token should be there
        assert!(blacklist.is_revoked("new-token"));
    }
}
