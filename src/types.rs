use bytes::Bytes;
use std::time::{Duration, SystemTime};

/// Synchronization policy for persistence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPolicy {
    /// Never sync to disk (fastest, least safe)
    Never,
    /// Sync every second (recommended)
    EverySecond,
    /// Sync after every write (slowest, safest)
    Always,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        Self::EverySecond
    }
}

/// Simplified database configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// How often data is synced to disk
    pub sync_policy: SyncPolicy,
    /// Default TTL for items (None means no default TTL)
    pub default_ttl: Option<Duration>,
}

/// Options for setting values
#[derive(Debug, Clone, Default)]
pub struct SetOptions {
    /// Time-to-live for this item
    pub ttl: Option<Duration>,
    /// Absolute expiration time
    pub expires_at: Option<SystemTime>,
}

impl SetOptions {
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            ttl: Some(ttl),
            expires_at: None,
        }
    }

    pub fn with_expiration(expires_at: SystemTime) -> Self {
        Self {
            ttl: None,
            expires_at: Some(expires_at),
        }
    }
}

/// Internal representation of a database item
#[derive(Debug, Clone)]
pub struct DbItem {
    /// The key
    pub key: Bytes,
    /// The value
    pub value: Bytes,
    /// Expiration time (if any)
    pub expires_at: Option<SystemTime>,
}

impl DbItem {
    pub fn new(key: impl Into<Bytes>, value: impl Into<Bytes>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            expires_at: None,
        }
    }

    pub fn with_expiration(
        key: impl Into<Bytes>,
        value: impl Into<Bytes>,
        expires_at: SystemTime,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            expires_at: Some(expires_at),
        }
    }

    pub fn with_ttl(key: impl Into<Bytes>, value: impl Into<Bytes>, ttl: Duration) -> Self {
        let expires_at = SystemTime::now() + ttl;
        Self::with_expiration(key, value, expires_at)
    }

    /// Check if this item has expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => SystemTime::now() > expires_at,
            None => false,
        }
    }

    /// Get remaining TTL
    pub fn ttl(&self) -> Option<Duration> {
        match self.expires_at {
            Some(expires_at) => expires_at.duration_since(SystemTime::now()).ok(),
            None => None,
        }
    }
}

/// Database statistics
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    /// Number of keys in the database
    pub key_count: usize,
    /// Number of items that have expired
    pub expired_count: u64,
    /// Total number of operations performed
    pub operations_count: u64,
}
