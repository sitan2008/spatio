<div align="center">

![SpatioLite Logo](assets/images/logo.png)

# SpatioLite

[![CI](https://github.com/spatiolite/spatiolite/workflows/CI/badge.svg)](https://github.com/spatiolite/spatiolite/actions)
[![Security](https://github.com/spatiolite/spatiolite/workflows/Security/badge.svg)](https://github.com/spatiolite/spatiolite/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/spatio_lite.svg)](https://crates.io/crates/spatio_lite)
[![Documentation](https://docs.rs/spatio_lite/badge.svg)](https://docs.rs/spatio_lite)

</div>

**SpatioLite** is a high-performance, embedded spatio-temporal database designed for modern applications that need to store and query location-based data with temporal components.

## Features

- **In-Memory Performance**: Fast reads and writes with optional persistence
- **Spatial Indexing**: Geohash, S2 cells, and R-tree indexing for geospatial data
- **Time-to-Live (TTL)**: Built-in expiration for temporal data
- **Thread-Safe**: Concurrent operations with atomic batches
- **Persistent Storage**: Append-only file (AOF) format with replay support
- **Geo-Spatial Features**: Point storage, trajectory tracking, and spatial queries
- **Advanced Geometry**: Full support for polygons and linestrings
- **Spatial Operations**: Point-in-polygon, intersections, distance calculations, and buffering
- **Standards Compliant**: WKT serialization and GeoJSON-compatible geometry types
- **Embeddable**: Simple API that integrates easily into any Rust application

## Installation

Add SpatioLite to your `Cargo.toml`:

```toml
[dependencies]
spatio_lite = "0.1"
```

## Quick Start

```rust
use spatio_lite::{
    Coordinate, Geometry, GeometryOps, LineString, LinearRing,
    Point, Polygon, SetOptions, SpatioLite
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory database
    let db = SpatioLite::memory()?;

    // Basic spatial point operations
    let nyc = Point::new(40.7128, -74.0060);
    db.insert_point("location:nyc", &nyc, None)?;

    // Insert with geohash indexing for spatial queries
    db.insert_point_with_geohash("cities", &nyc, 8, b"New York City", None)?;

    // Advanced geometry support - Polygons
    let park_coords = vec![
        Coordinate::new(-73.9733, 40.7644), // SW corner
        Coordinate::new(-73.9500, 40.7644), // SE corner
        Coordinate::new(-73.9500, 40.7997), // NE corner
        Coordinate::new(-73.9733, 40.7997), // NW corner
        Coordinate::new(-73.9733, 40.7644), // Close the ring
    ];
    let park_ring = LinearRing::new(park_coords)?;
    let central_park = Polygon::new(park_ring);
    db.insert_polygon("parks", &central_park, b"Central Park", None)?;

    // LineString geometries for routes and paths
    let route_coords = vec![
        Coordinate::new(-73.9857, 40.7484), // Times Square
        Coordinate::new(-73.9867, 40.7505),
        Coordinate::new(-73.9877, 40.7526), // Columbus Circle
    ];
    let broadway = LineString::new(route_coords)?;
    db.insert_linestring("streets", &broadway, b"Broadway", None)?;

    // Spatial queries with geometry support
    let test_point = Coordinate::new(-73.9650, 40.7820);
    let containing_parks = db.geometries_containing_point("parks", &test_point)?;
    println!("Found {} parks containing the point", containing_parks.len());

    // Geometry operations and analysis
    let buffer_zone = GeometryOps::buffer_point(&test_point, 0.005, 16)?;
    db.insert_polygon("zones", &buffer_zone, b"Safety Zone", None)?;

    // WKT serialization
    let point_geom = Geometry::Point(test_point);
    println!("Point WKT: {}", point_geom.to_wkt());

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

## Use Cases

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

### Geospatial Analysis & Geometry
Store and query complex geometric shapes:

```rust
use spatio_lite::{Coordinate, Geometry, GeometryOps, LineString, LinearRing, Polygon};

// Create complex polygons with holes
let building_exterior = vec![
    Coordinate::new(-73.9850, 40.7580),
    Coordinate::new(-73.9820, 40.7580),
    Coordinate::new(-73.9820, 40.7610),
    Coordinate::new(-73.9850, 40.7610),
    Coordinate::new(-73.9850, 40.7580), // Close the ring
];

let courtyard_hole = vec![
    Coordinate::new(-73.9840, 40.7590),
    Coordinate::new(-73.9830, 40.7590),
    Coordinate::new(-73.9830, 40.7600),
    Coordinate::new(-73.9840, 40.7600),
    Coordinate::new(-73.9840, 40.7590), // Close the hole
];

let exterior_ring = LinearRing::new(building_exterior)?;
let hole_ring = LinearRing::new(courtyard_hole)?;
let building = Polygon::with_holes(exterior_ring, vec![hole_ring]);

// Store polygon with spatial indexing
db.insert_polygon("buildings", &building, b"Office Complex", None)?;

// Point-in-polygon queries
let query_point = Coordinate::new(-73.9835, 40.7595);
let containing_buildings = db.geometries_containing_point("buildings", &query_point)?;

// Bounding box spatial queries
let bbox_min = Coordinate::new(-74.0000, 40.7500);
let bbox_max = Coordinate::new(-73.9500, 40.8000);
let geometries_in_area = db.geometries_within_bounds("buildings", &bbox_min, &bbox_max)?;

// Calculate areas and perform geometric operations
let total_building_area = db.total_polygon_area("buildings")?;
let buffer_zone = GeometryOps::buffer_point(&query_point, 0.001, 12)?; // 100m radius

// WKT serialization for interoperability
let geom = Geometry::Polygon(building);
println!("Building WKT: {}", geom.to_wkt());
```

### Real-time Analytics
Process streaming location data with automatic cleanup:

```rust
// Store temporary analytics data with TTL
let ttl_opts = SetOptions::with_ttl(Duration::from_hours(1));
db.insert("analytics:traffic:route_66", b"congestion_level:high", Some(ttl_opts))?;
```

## Architecture

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

## Performance

SpatioLite is optimized for high-throughput scenarios:

- **1M+ operations/second** for in-memory workloads
- **Sub-millisecond** query latency for indexed data
- **Memory-efficient** spatial indexing
- **Atomic batch operations** for consistency without traditional ACID overhead

## Configuration

```rust
use spatio_lite::{Config, SyncPolicy};

let mut config = Config::default();
config.sync_policy = SyncPolicy::EverySecond;
config.auto_shrink_percentage = 100;
config.max_dimensions = 3; // For 3D spatial data

let db = SpatioLite::memory()?;
db.set_config(config)?;
```

## Development Status

SpatioLite is currently in **early development** (v0.1.x). The core functionality is working, but the API may change before v1.0.

### Implemented
- [x] In-memory key-value storage
- [x] Atomic operations and batches
- [x] TTL/expiration support
- [x] AOF persistence with replay and auto-compaction
- [x] Spatial point operations
- [x] Geohash and S2 cell indexing
- [x] Trajectory tracking and queries
- [x] Nearest neighbor search
- [x] Thread-safe operations
- [x] Complex geometry support (Polygons, LineStrings)
- [x] Advanced spatial queries (contains, intersects, within bounds)
- [x] Spatial operations (buffer, distance, area calculations)
- [x] WKT serialization and geometry persistence
- [x] Polygon with holes support
- [x] Comprehensive test suite
- [x] Benchmarking suite

### In Progress
- [ ] Index management API
- [ ] Performance optimizations
- [ ] GeoJSON import/export

### Planned
- [ ] Spatial joins and complex queries
- [ ] Geometry validation and repair
- [ ] Coordinate reference system support
- [ ] Compression for AOF files
- [ ] Backup/restore utilities
- [ ] Monitoring and metrics
- [ ] Query language (SpatioQL)

## Documentation

- [API Documentation](https://docs.rs/spatio_lite) (Generated from code)
- [Examples](src/main.rs) - Comprehensive spatial demo
- [Benchmarks](benches/) - Performance testing suite
- [Assets](assets/) - Logo and branding materials

### Logo Usage

The SpatioLite logo is available at `assets/images/logo.png`. When using the logo in your own projects or documentation:

```html
<!-- For web usage -->
<img src="https://raw.githubusercontent.com/pkvartsianyi/SpatioLite/main/assets/images/logo.png" alt="SpatioLite Logo" width="200">
```

```markdown
<!-- For Markdown documentation -->
![SpatioLite Logo](https://raw.githubusercontent.com/pkvartsianyi/SpatioLite/main/assets/images/logo.png)
```

Please refer to the [assets directory](assets/) for usage guidelines and additional branding materials.

## Contributing

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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

SpatioLite is inspired by:
- [BuntDB](https://github.com/tidwall/buntdb) - Fast embeddable in-memory key/value database in Go
- [Redis](https://redis.io/) - For its excellent performance characteristics
- [PostGIS](https://postgis.net/) - For spatial database operations

## Contact

- **Issues**: [GitHub Issues](https://github.com/pkvartsianyi/SpatioLite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/pkvartsianyi/SpatioLite/discussions)

---

**Built with Rust**
