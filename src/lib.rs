//! # SpatioLite - An embedded spatio-temporal database
//!
//! SpatioLite is a high-performance, embedded spatio-temporal database designed for modern applications
//! that need to store and query location-based data with temporal components.
//!
//! ## Features
//!
//! - **In-Memory Performance**: Fast reads and writes with optional persistence
//! - **Spatial Indexing**: Geohash, S2 cells, and R-tree indexing for geospatial data
//! - **Time-to-Live (TTL)**: Built-in expiration for temporal data
//! - **Thread-Safe**: Concurrent operations with atomic batches
//! - **Persistent Storage**: Append-only file (AOF) format with replay support
//! - **Geo-Spatial Features**: Point storage, trajectory tracking, and spatial queries
//! - **Embeddable**: Simple API that integrates easily into any Rust application
//!
//! ## Quick Start
//!
//! ```rust
//! use spatio::{Point, SetOptions, Spatio};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an in-memory database
//! let db = Spatio::memory()?;
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
//!
//! ## Advanced Geometry Operations
//!
//! ```rust
//! use spatio::{Spatio, geometry::{Coordinate, Polygon, LinearRing, GeometryOps}};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let db = Spatio::memory()?;
//!
//! // Create a polygon (e.g., a park boundary)
//! let park_coords = vec![
//!     Coordinate::new(-73.9733, 40.7644), // SW corner
//!     Coordinate::new(-73.9500, 40.7644), // SE corner
//!     Coordinate::new(-73.9500, 40.7997), // NE corner
//!     Coordinate::new(-73.9733, 40.7997), // NW corner
//!     Coordinate::new(-73.9733, 40.7644), // Close the ring
//! ];
//! let park_ring = LinearRing::new(park_coords)?;
//! let central_park = Polygon::new(park_ring);
//!
//! // Store the polygon
//! db.insert_polygon("parks", &central_park, b"Central Park", None)?;
//!
//! // Create a buffer zone around a point
//! let center = Coordinate::new(-73.9857, 40.7484);
//! let buffer = GeometryOps::buffer_point(&center, 0.005, 16)?;
//! db.insert_polygon("zones", &buffer, b"Safety Zone", None)?;
//!
//! // Spatial queries - find geometries containing a point
//! let test_point = Coordinate::new(-73.9650, 40.7820);
//! let containing = db.geometries_containing_point("parks", &test_point)?;
//! println!("Found {} parks containing the point", containing.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Trajectory Tracking
//!
//! ```rust
//! use spatio::{Spatio, Point, SetOptions};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let db = Spatio::memory()?;
//!
//! // Track a vehicle's movement over time
//! let vehicle_path = vec![
//!     (Point::new(40.7128, -74.0060), 1640995200), // Start: Financial District
//!     (Point::new(40.7180, -74.0020), 1640995260), // Move north (1 min later)
//!     (Point::new(40.7230, -73.9980), 1640995320), // Continue north (2 min later)
//!     (Point::new(40.7484, -73.9857), 1640995380), // End: Times Square (3 min later)
//! ];
//!
//! // Store trajectory with TTL
//! let ttl_opts = Some(SetOptions::with_ttl(Duration::from_secs(3600)));
//! db.insert_trajectory("vehicle:truck001", &vehicle_path, ttl_opts)?;
//!
//! // Query trajectory for specific time range
//! let path_segment = db.query_trajectory("vehicle:truck001", 1640995200, 1640995320)?;
//! println!("Retrieved {} waypoints for first 2 minutes", path_segment.len());
//! # Ok(())
//! # }
//! ```

pub mod batch;
pub mod db;
pub mod error;
pub mod geometry;
pub mod index;
pub mod persistence;
pub mod spatial;

pub mod types;

// Re-export core database types
pub use db::DB;
pub use error::{Result, SpatioLiteError};

// Main database type alias
pub type Spatio = DB;

// Error type alias for consistency
pub type SpatioError = SpatioLiteError;

// Re-export spatial types and utilities
pub use spatial::{
    BoundingBox, CoordinateSystem, GeohashUtils, Point, S2Utils, SpatialAnalysis, SpatialKey,
};

// Re-export geometry types
pub use geometry::{Coordinate, Geometry, GeometryOps, LineString, LinearRing, Polygon};

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
        AtomicBatch, BoundingBox, Coordinate, Geometry, Point, Result, SetOptions, SpatioLite,
        SpatioLiteError,
    };
    pub use std::time::Duration;
}
