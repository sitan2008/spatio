# SpatioLite

[![CI](https://github.com/spatiolite/spatiolite/workflows/CI/badge.svg)](https://github.com/spatiolite/spatiolite/actions)
[![Security](https://github.com/spatiolite/spatiolite/workflows/Security/badge.svg)](https://github.com/spatiolite/spatiolite/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/spatio_lite.svg)](https://crates.io/crates/spatio_lite)
[![Documentation](https://docs.rs/spatio_lite/badge.svg)](https://docs.rs/spatio_lite)

**SpatioLite** is a high-performance, embedded spatio-temporal database designed for modern applications that need to store and query location-based data with temporal components.

## 🚀 Features

- **🧠 In-Memory Performance**: Fast reads and writes with optional persistence
- **🌍 Spatial Indexing**: Geohash, S2 cells, and R-tree indexing for geospatial data
- **⏰ Time-to-Live (TTL)**: Built-in expiration for temporal data
- **🔒 Thread-Safe**: Concurrent operations with atomic batches
- **💾 Persistent Storage**: Append-only file (AOF) format with replay support
- **📍 Geo-Spatial Features**: Point storage, trajectory tracking, and spatial queries
- **🔧 Embeddable**: Simple API that integrates easily into any Rust application

## 📦 Installation

Add SpatioLite to your `Cargo.toml`:

```toml
[dependencies]
spatio_lite = "0.1"
```

## 🏃‍♂️ Quick Start

```rust
use spatio_lite::{Point, SetOptions, SpatioLite};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory database
    let db = SpatioLite::memory()?;

    // Spatial point operations
    let nyc = Point::new(40.7128, -74.0060);
    db.insert_point("location:nyc", &nyc, None)?;

    // Insert with geohash indexing for spatial queries
    db.insert_point_with_geohash("cities", &nyc, 8, b"New York City", None)?;

    // Atomic batch operations
    db.atomic(|batch| {
        batch.insert("sensor:temp", b"22.5°C", None)?;
        batch.insert("sensor:humidity", b"65%", None)?;
        batch.insert("sensor:pressure", b"1013.25hPa", None)?;
        Ok(())
    })?;

    // TTL support for temporary data
    let opts = SetOptions::with_ttl(Duration::from_secs(300));
    db.insert("temp:reading", b"sensor_data", Some(opts))?;

    // Trajectory tracking
    let trajectory = vec![
        (Point::new(40.7128, -74.0060), 1640995200),
        (Point::new(40.7150, -74.0040), 1640995230),
        (Point::new(40.7172, -74.0020), 1640995260),
    ];
    db.insert_trajectory("drone:001", &trajectory, None)?;

    // Spatial queries
    let nearby = db.find_nearest_neighbors("cities", &nyc, 10000.0, 10)?;
    println!("Found {} nearby cities", nearby.len());

    // Persistent database with AOF replay
    let persistent_db = SpatioLite::open("my_data.aof")?;
    persistent_db.insert("persistent:key", b"persistent_value", None)?;
    persistent_db.sync()?; // Force sync to disk

    Ok(())
}
```

## 🌐 Use Cases

### IoT & Sensor Networks
Store sensor readings with location and timestamp information:

```rust
// Store temperature sensor with spatial indexing
let sensor_location = Point::new(40.7128, -74.0060);
db.insert_point_with_geohash("sensors", &sensor_location, 8, b"temp:22.5,humidity:65", None)?;

// Find nearby sensors within 1km
let nearby_sensors = db.find_nearest_neighbors("sensors", &sensor_location, 1000.0, 10)?;
```

### Vehicle Tracking
Track vehicles, drones, or any moving objects:

```rust
// UAV trajectory tracking with timestamps
let trajectory = vec![
    (Point::new(40.7128, -74.0060), 1640995200), // Start position
    (Point::new(40.7150, -74.0040), 1640995230), // 30 seconds later
    (Point::new(40.7172, -74.0020), 1640995260), // 1 minute later
];
db.insert_trajectory("uav:alpha", &trajectory, None)?;

// Query trajectory between timestamps
let path = db.query_trajectory("uav:alpha", 1640995200, 1640995260)?;
```

### Real-time Analytics
Process streaming location data with automatic cleanup:

```rust
// Store temporary analytics data with TTL
let ttl_opts = SetOptions::with_ttl(Duration::from_hours(1));
db.insert("analytics:traffic:route_66", b"congestion_level:high", Some(ttl_opts))?;
```

## 🏗️ Architecture

SpatioLite is designed with a simple but powerful architecture:

```
┌─────────────────┐
│   Application   │
└─────────┬───────┘
          │
┌─────────▼───────┐
│ SpatioLite API  │
└─────────┬───────┘
          │
┌─────────▼───────┐    ┌─────────────────┐
│ Atomic Batches  │◄───┤ Transaction     │
└─────────┬───────┘    │ Management      │
          │            └─────────────────┘
┌─────────▼───────┐
│ Storage Engine  │
└─────┬───┬───────┘
      │   │
┌─────▼───▼───────┐    ┌─────────────────┐
│ Memory Store    │    │ Spatial Indexes │
│ (B-Tree)        │    │ (R-Tree)        │
└─────────┬───────┘    └─────────────────┘
          │
┌─────────▼───────┐
│ AOF Persistence │
└─────────────────┘
```

## ⚡ Performance

SpatioLite is optimized for high-throughput scenarios:

- **1M+ operations/second** for in-memory workloads
- **Sub-millisecond** query latency for indexed data
- **Memory-efficient** spatial indexing
- **Atomic batch operations** for consistency without traditional ACID overhead

## 🔧 Configuration

```rust
use spatio_lite::{Config, SyncPolicy};

let mut config = Config::default();
config.sync_policy = SyncPolicy::EverySecond;
config.auto_shrink_percentage = 100;
config.max_dimensions = 3; // For 3D spatial data

let db = SpatioLite::memory()?;
db.set_config(config)?;
```

## 🛠️ Development Status

SpatioLite is currently in **early development** (v0.1.x). The core functionality is working, but the API may change before v1.0.

### ✅ Implemented
- [x] In-memory key-value storage
- [x] Atomic operations and batches
- [x] TTL/expiration support
- [x] AOF persistence with replay
- [x] Spatial point operations
- [x] Geohash and S2 cell indexing
- [x] Trajectory tracking and queries
- [x] Nearest neighbor search
- [x] Thread-safe operations
- [x] Comprehensive test suite
- [x] Benchmarking suite

### 🚧 In Progress
- [ ] Advanced spatial queries (intersects, within)
- [ ] Index management API
- [ ] AOF auto-compaction
- [ ] Performance optimizations

### 📋 Planned
- [ ] Complex geometry support
- [ ] Compression for AOF files
- [ ] Backup/restore utilities
- [ ] Monitoring and metrics
- [ ] Query language (SpatioQL)

## 📖 Documentation

- [API Documentation](https://docs.rs/spatio_lite) (Generated from code)
- [Examples](src/main.rs) - Comprehensive spatial demo
- [Benchmarks](benches/) - Performance testing suite

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/pkvartsianyi/SpatioLite.git
cd SpatioLite

# Run all tests
cargo test

# Run the spatial demo
cargo run

# Run benchmarks
cargo bench

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

SpatioLite is inspired by:
- [BuntDB](https://github.com/tidwall/buntdb) - Fast embeddable in-memory key/value database in Go
- [Redis](https://redis.io/) - For its excellent performance characteristics
- [PostGIS](https://postgis.net/) - For spatial database operations

## 📞 Contact

- **Issues**: [GitHub Issues](https://github.com/pkvartsianyi/SpatioLite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/pkvartsianyi/SpatioLite/discussions)

---

**Built with ❤️ in Rust**
