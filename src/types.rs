use crate::error::Result;
use bytes::Bytes;
use smallvec::SmallVec;

use std::time::{Duration, SystemTime};

/// Synchronization policy for persistence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPolicy {
    /// Never sync to disk (fastest, least safe)
    Never,
    /// Sync every second (recommended balance)
    EverySecond,
    /// Sync after every write (slowest, safest)
    Always,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        Self::EverySecond
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// How often data is synced to disk
    pub sync_policy: SyncPolicy,

    /// Percentage threshold for auto-shrinking AOF file
    pub auto_shrink_percentage: u32,

    /// Minimum size before auto-shrink kicks in
    pub auto_shrink_min_size: u64,

    /// Disable automatic shrinking
    pub auto_shrink_disabled: bool,

    /// Maximum number of dimensions for spatial indexing
    pub max_dimensions: usize,

    /// Default TTL for items (None means no default TTL)
    pub default_ttl: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_policy: SyncPolicy::default(),
            auto_shrink_percentage: 100,
            auto_shrink_min_size: 32 * 1024 * 1024, // 32MB
            auto_shrink_disabled: false,
            max_dimensions: 20,
            default_ttl: None,
        }
    }
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

/// Options for creating indexes
#[derive(Debug, Clone, Default)]
pub struct IndexOptions {
    /// Case insensitive key matching for patterns
    pub case_insensitive: bool,
    /// Whether this is a unique index
    pub unique: bool,
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
    /// Whether this item is keyless (used for scanning)
    pub keyless: bool,
}

impl DbItem {
    pub fn new(key: impl Into<Bytes>, value: impl Into<Bytes>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            expires_at: None,
            keyless: false,
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
            keyless: false,
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
            Some(expires_at) => SystemTime::now().duration_since(expires_at).ok(),
            None => None,
        }
    }

    /// Create a keyless item for scanning
    pub fn keyless(value: impl Into<Bytes>) -> Self {
        Self {
            key: Bytes::new(),
            value: value.into(),
            expires_at: None,
            keyless: true,
        }
    }
}

/// Spatial rectangle for indexing
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    /// Minimum coordinates for each dimension
    pub min: SmallVec<[f64; 4]>,
    /// Maximum coordinates for each dimension
    pub max: SmallVec<[f64; 4]>,
}

impl Rect {
    pub fn new(min: Vec<f64>, max: Vec<f64>) -> Result<Self> {
        if min.len() != max.len() {
            return Err(crate::error::SpatioLiteError::Invalid);
        }

        for (min_val, max_val) in min.iter().zip(max.iter()) {
            if min_val > max_val {
                return Err(crate::error::SpatioLiteError::Invalid);
            }
        }

        Ok(Self {
            min: min.into(),
            max: max.into(),
        })
    }

    pub fn point(coords: Vec<f64>) -> Self {
        Self {
            min: coords.clone().into(),
            max: coords.into(),
        }
    }

    pub fn dimensions(&self) -> usize {
        self.min.len()
    }

    pub fn contains_point(&self, point: &[f64]) -> bool {
        if point.len() != self.dimensions() {
            return false;
        }

        for (i, &point_val) in point.iter().enumerate().take(self.dimensions()) {
            if point_val < self.min[i] || point_val > self.max[i] {
                return false;
            }
        }
        true
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        if self.dimensions() != other.dimensions() {
            return false;
        }

        for (i, (&self_max, &self_min)) in self
            .max
            .iter()
            .zip(self.min.iter())
            .enumerate()
            .take(self.dimensions())
        {
            if self_max < other.min[i] || self_min > other.max[i] {
                return false;
            }
        }
        true
    }
}

/// Type alias for custom comparison functions
pub type LessFunc = Box<dyn Fn(&[u8], &[u8]) -> bool + Send + Sync>;

/// Type alias for spatial rectangle extraction functions
pub type RectFunc = Box<dyn Fn(&[u8]) -> Result<Rect> + Send + Sync>;

/// Index type enumeration
#[derive(Debug)]
pub enum IndexType {
    /// B-tree index for ordered data
    BTree,
    /// R-tree index for spatial data
    RTree,
}

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read committed (default)
    ReadCommitted,
    /// Snapshot isolation
    Snapshot,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        Self::ReadCommitted
    }
}

/// Statistics about the database
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    /// Number of keys in the database
    pub key_count: u64,
    /// Number of indexes
    pub index_count: u64,
    /// Size of the AOF file in bytes
    pub aof_size: u64,
    /// Number of expired items cleaned up
    pub expired_count: u64,
    /// Number of disk flushes performed
    pub flush_count: u64,
    /// Number of shrink operations performed
    pub shrink_count: u64,
}
