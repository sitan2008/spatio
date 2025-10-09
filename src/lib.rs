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
