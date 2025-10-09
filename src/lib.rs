//! SpatioLite - An embedded spatio-temporal database
//!
//! SpatioLite is an embedded database optimized for spatio-temporal data with support for:
//! - In-memory storage with optional persistence
//! - Multi-dimensional spatial indexing using R-trees
//! - B-tree indexing for ordered data
//! - TTL/expiration support
//! - Atomic operations and batches
//! - Append-only file (AOF) persistence

pub mod batch;
pub mod db;
pub mod error;
pub mod index;
pub mod persistence;
pub mod types;

// Re-export main types for convenience
pub use batch::AtomicBatch;
pub use db::DB;
pub use error::{Result, SpatioLiteError};
pub use types::{Config, IndexOptions, Rect, SetOptions, SyncPolicy};

/// Main SpatioLite database - alias for DB
pub type SpatioLite = DB;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_basic_usage() {
        let db = SpatioLite::memory().unwrap();

        // Single atomic insert
        db.insert("location:123", &b"lat:40.7128,lon:-74.0060"[..], None)
            .unwrap();

        // Get the value
        let value = db.get("location:123").unwrap().unwrap();
        assert_eq!(value, &b"lat:40.7128,lon:-74.0060"[..]);

        // Delete
        let deleted = db.delete("location:123").unwrap().unwrap();
        assert_eq!(deleted, &b"lat:40.7128,lon:-74.0060"[..]);

        // Should be gone now
        assert!(db.get("location:123").unwrap().is_none());
    }

    #[test]
    fn test_atomic_batch() {
        let db = SpatioLite::memory().unwrap();

        // Atomic batch of operations
        db.atomic(|batch| {
            batch.insert("uav:1", &b"40.7128,-74.0060,100"[..], None)?;
            batch.insert("uav:2", &b"40.7589,-73.9851,150"[..], None)?;
            batch.insert("uav:3", &b"40.6892,-74.0445,200"[..], None)?;
            Ok(())
        })
        .unwrap();

        // All items should be present
        assert!(db.get("uav:1").unwrap().is_some());
        assert!(db.get("uav:2").unwrap().is_some());
        assert!(db.get("uav:3").unwrap().is_some());
    }

    #[test]
    fn test_ttl() {
        let db = SpatioLite::memory().unwrap();

        // Insert with TTL
        let opts = SetOptions::with_ttl(Duration::from_millis(100));
        db.insert("temp:data", &b"expires_soon"[..], Some(opts))
            .unwrap();

        // Should exist initially
        assert!(db.get("temp:data").unwrap().is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired now
        assert!(db.get("temp:data").unwrap().is_none());
    }

    #[test]
    #[ignore] // TODO: Enable when AOF loading is implemented
    fn test_persistence() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create database with persistence
        {
            let db = SpatioLite::open(path).unwrap();
            db.insert("persistent:key", &b"persistent_value"[..], None)
                .unwrap();
            db.sync().unwrap();
        } // Database closes here

        // Reopen and verify data persisted
        {
            let db = SpatioLite::open(path).unwrap();
            let value = db.get("persistent:key").unwrap().unwrap();
            assert_eq!(value, &b"persistent_value"[..]);
        }
    }

    #[test]
    fn test_spatial_rect() {
        // Test rectangle creation and operations
        let rect1 = Rect::new(vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();
        let rect2 = Rect::new(vec![5.0, 5.0], vec![15.0, 15.0]).unwrap();

        assert!(rect1.intersects(&rect2));
        assert!(rect1.contains_point(&[5.0, 5.0]));
        assert!(!rect1.contains_point(&[15.0, 15.0]));

        let point = Rect::point(vec![7.5, 7.5]);
        assert!(rect1.intersects(&point));
        assert!(rect2.intersects(&point));
    }

    #[test]
    fn test_config() {
        let mut config = Config::default();
        config.sync_policy = SyncPolicy::Always;
        config.auto_shrink_disabled = true;
        config.max_dimensions = 3;

        let db = SpatioLite::memory().unwrap();
        db.set_config(config.clone()).unwrap();

        let retrieved_config = db.config().unwrap();
        assert_eq!(retrieved_config.sync_policy, SyncPolicy::Always);
        assert!(retrieved_config.auto_shrink_disabled);
        assert_eq!(retrieved_config.max_dimensions, 3);
    }

    #[test]
    fn test_stats() {
        let db = SpatioLite::memory().unwrap();

        let initial_stats = db.stats().unwrap();
        assert_eq!(initial_stats.key_count, 0);

        // Add some data
        db.insert("key1", &b"value1"[..], None).unwrap();
        db.insert("key2", &b"value2"[..], None).unwrap();

        let stats = db.stats().unwrap();
        assert_eq!(stats.key_count, 2);
    }
}
