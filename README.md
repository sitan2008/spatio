# SpatioLite

[![CI](https://github.com/spatiolite/spatiolite/workflows/CI/badge.svg)](https://github.com/spatiolite/spatiolite/actions)
[![Security](https://github.com/spatiolite/spatiolite/workflows/Security/badge.svg)](https://github.com/spatiolite/spatiolite/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/spatio_lite.svg)](https://crates.io/crates/spatio_lite)
[![Documentation](https://docs.rs/spatio_lite/badge.svg)](https://docs.rs/spatio_lite)

**SpatioLite** is a high-performance, embedded spatio-temporal database designed for modern applications that need to store and query location-based data with temporal components.

## 🚀 Features

- **🧠 In-Memory Performance**: Fast reads and writes with optional persistence
- **🌍 Spatial Indexing**: Multi-dimensional R-tree indexing for up to 20 dimensions
- **⏰ Time-to-Live (TTL)**: Built-in expiration for temporal data
- **🔒 Thread-Safe**: Concurrent operations with atomic batches
- **💾 Persistent Storage**: Append-only file (AOF) format for durability
- **🔧 Embeddable**: Simple API that integrates easily into any Rust application

## 📦 Installation

Add SpatioLite to your `Cargo.toml`:

```toml
[dependencies]
spatio_lite = "0.1"
```

## 🏃‍♂️ Quick Start

```rust
use spatio_lite::{SpatioLite, SetOptions};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory database
    let db = SpatioLite::memory()?;

    // Single atomic operations
    db.insert("drone:001", b"lat:40.7128,lon:-74.0060,alt:100", None)?;

    // Get the data
    let location = db.get("drone:001")?.unwrap();
    println!("Location: {}", String::from_utf8_lossy(&location));

    // Atomic batch operations
    db.atomic(|batch| {
        batch.insert("sensor:temp", b"22.5°C", None)?;
        batch.insert("sensor:humidity", b"65%", None)?;
        batch.insert("sensor:pressure", b"1013.25hPa", None)?;
        Ok(())
    })?;

    // TTL support
    let opts = SetOptions::with_ttl(Duration::from_secs(300));
    db.insert("temp:reading", b"sensor_data", Some(opts))?;

    // Persistent database
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
// Store temperature sensor data with coordinates
db.insert("sensor:temp:001", b"lat:40.7128,lon:-74.0060,temp:22.5,ts:1640995200", None)?;

// Query nearby sensors
// TODO: Spatial queries coming soon
```

### Vehicle Tracking
Track vehicles, drones, or any moving objects:

```rust
// UAV position tracking
db.atomic(|batch| {
    batch.insert("uav:alpha:pos", b"40.7128,-74.0060,100", None)?;
    batch.insert("uav:alpha:heading", b"245.7", None)?;
    batch.insert("uav:alpha:speed", b"45.2", None)?;
    Ok(())
})?;
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
- [x] AOF persistence format
- [x] Basic spatial indexing framework
- [x] Thread-safe operations
- [x] Comprehensive test suite

### 🚧 In Progress
- [ ] Index creation API
- [ ] AOF loading/replay
- [ ] Auto-shrinking/compaction
- [ ] Performance optimizations

### 📋 Planned
- [ ] Geo-coordinate helpers
- [ ] S2/Geohash integration
- [ ] Compression support
- [ ] Backup/restore utilities
- [ ] Monitoring and metrics

## 📖 Documentation

- [API Documentation](Coming Soon)
- [User Guide] (Coming Soon)
- [Examples](examples/) (Coming Soon)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/spatiolite/spatiolite.git
cd spatiolite

# Run tests
cargo test

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

- **Issues**: [GitHub Issues](https://github.com/pkvartsianyi/spatio-lite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/pkvartsianyi/spatio-lite/discussions)

---

**Built with ❤️ in Rust**
