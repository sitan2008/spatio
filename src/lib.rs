//! # SpatioLite - An embedded spatio-temporal database
//!
//! SpatioLite is a high-performance, embedded spatio-temporal database designed for modern applications
//! that need to store and query location-based data with temporal components.
//!
//! ## Features
//!
//! - **🧠 In-Memory Performance**: Fast reads and writes with optional persistence
//! - **🌍 Spatial Indexing**: Geohash, S2 cells, and R-tree indexing for geospatial data
//! - **⏰ Time-to-Live (TTL)**: Built-in expiration for temporal data
//! - **🔒 Thread-Safe**: Concurrent operations with atomic batches
//! - **💾 Persistent Storage**: Append-only file (AOF) format with replay support
//! - **📍 Geo-Spatial Features**: Point storage, trajectory tracking, and spatial queries
//! - **🔧 Embeddable**: Simple API that integrates easily into any Rust application
//!
//! ## Quick Start
//!
//! ```rust
//! use spatio_lite::{Point, SetOptions, SpatioLite};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an in-memory database
//! let db = SpatioLite::memory()?;
//!
//! // Spatial point operations
//! let nyc = Point::new(40.7128, -74.0060);
//! db.insert_point("location:nyc", &nyc, None)?;
//!
//! // Insert with geohash indexing for spatial queries
//! db.insert_point_with_geohash("cities", &nyc, 8, b"New York City", None)?;
//!
//! // Atomic batch operations
//! db.atomic(|batch| {
//!     batch.insert("sensor:temp", b"22.5C", None)?;
//!     batch.insert("sensor:humidity", b"65%", None)?;
//!     Ok(())
//! })?;
//!
//! // TTL support for temporary data
//! let opts = SetOptions::with_ttl(Duration::from_secs(300));
//! db.insert("temp:reading", b"sensor_data", Some(opts))?;
//!
//! // Spatial queries
//! let nearby = db.find_nearest_neighbors("cities", &nyc, 10000.0, 10)?;
//! println!("Found {} nearby cities", nearby.len());
//! # Ok(())
//! # }
//! ```

pub mod batch;
pub mod db;
pub mod error;
pub mod index;
pub mod persistence;
pub mod spatial;

pub mod types;

// Re-export core database types
pub use db::DB;
pub use error::{Result, SpatioLiteError};

// Re-export spatial types and utilities
pub use spatial::{
    BoundingBox, CoordinateSystem, GeohashUtils, Point, S2Utils, SpatialAnalysis, SpatialKey,
};

// Re-export batch and transaction types
pub use batch::AtomicBatch;

// Re-export configuration and option types
pub use types::{Config, DbStats, IndexOptions, Rect, SetOptions, SyncPolicy};

// Re-export spatial types from db module
pub use db::SpatialStats;

// Re-export persistence types for advanced usage
pub use persistence::{AOFCommand, AOFFile};

/// Main SpatioLite database - alias for DB
pub type SpatioLite = DB;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common imports
pub mod prelude {
    pub use crate::{
        AtomicBatch, BoundingBox, Point, Result, SetOptions, SpatioLite, SpatioLiteError,
    };
    pub use std::time::Duration;
}
