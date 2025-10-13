<p align="center">
    <a href="https://github.com/pkvartsianyi/spatio">
        <img src="assets/images/logo-min.png" height="60" alt="Spatio Logo">
    </a>
</p>

<h1 align="center">Spatio</h1>

<p align="center">
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT">
  </a>
  <a href="https://crates.io/crates/spatio">
    <img src="https://img.shields.io/crates/v/spatio.svg" alt="Crates.io">
  </a>
  <a href="https://docs.rs/spatio">
    <img src="https://img.shields.io/badge/Docs-Available-blue.svg" alt="Documentation">
  </a>
</p>


**Spatio** is a high-performance, embedded spatio-temporal database designed for modern applications that need to store and query location-based data with temporal components.

## Features

- **Fast Key-Value Storage**: High-performance in-memory operations with optional persistence
- **Automatic Spatial Indexing**: Points are automatically indexed for efficient spatial queries
- **Trajectory Tracking**: Store and query movement paths over time
- **TTL Support**: Built-in data expiration for temporary data
- **Atomic Operations**: Batch multiple operations for data consistency
- **Thread-Safe**: Concurrent read/write access without blocking
- **Simple API**: Clean, focused interface that's easy to learn and use
- **Embedded**: No external dependencies or setup required

## Installation

Add Spatio to your `Cargo.toml`:

```toml
[dependencies]
spatio = "0.1"
```

## Quick Start

```rust
use spatio::{
    Coordinate, Geometry, GeometryOps, LineString, LinearRing,
    Point, Polygon, SetOptions, Spatio
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory database
    let db = Spatio::memory()?;

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

    // Create spatial index for faster queries (optional - done automatically)
    db.create_spatial_index("cities")?;

    // Persistent database with AOF replay
    let persistent_db = Spatio::open("my_data.db")?;
    persistent_db.insert("persistent:key", b"persistent_value", None)?;
    persistent_db.sync()?; // Force sync to disk

    // Note: Both .db and .aof extensions are supported
    // let db_alt = Spatio::open("data.aof")?;  // Also works

    Ok(())
}
```

## Examples

Spatio includes several focused example programs demonstrating different use cases:

### Getting Started
```bash
cargo run --example getting_started
```
A gentle introduction covering basic operations, spatial indexing, and TTL features.

### Spatial Queries
```bash
cargo run --example spatial_queries
```
Advanced spatial queries including distance-based searches, bounding boxes, and geohash analysis.

### Trajectory Tracking
```bash
cargo run --example trajectory_tracking
```
Vehicle tracking, drone paths, pedestrian routes, geofencing, and real-time position updates.

### Comprehensive Demo
```bash
cargo run --example comprehensive_demo
```
Complete feature showcase including all geometry types, spatial operations, and analysis tools.

All examples are self-contained and can be run independently to explore different aspects of Spatio's capabilities.

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
use spatio::{Coordinate, Geometry, GeometryOps, LineString, LinearRing, Polygon};

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

Spatio is designed with a simple but powerful architecture:

```
┌─────────────────┐
│   Application   │
└─────────┬───────┘
          │
┌─────────▼───────┐
│   Spatio API    │
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

Spatio is optimized for high-throughput scenarios with excellent performance characteristics:

- **High Throughput**: 1.5M+ operations/second for basic operations
- **Spatial Performance**: 2M+ spatial insertions/second with automatic indexing
- **Low Latency**: Microsecond-level latency for individual operations
- **Spatial Queries**: Sub-millisecond nearest neighbor search with R-tree indexing
- **Concurrency**: Thread-safe operations with minimal contention
- **Persistence**: Fast AOF writes with configurable sync policies
- **Automatic Optimization**: Spatial indexes created automatically for optimal performance

## Spatial Features

Spatio automatically indexes geographic points for efficient spatial queries:

### Automatic Indexing

Points are automatically indexed when inserted:

```rust
// Points are automatically indexed for spatial queries
db.insert_point("cities", &nyc, b"New York", None)?;
db.insert_point("cities", &london, b"London", None)?;

// Find nearby points within 1000km
let nearby = db.find_nearby("cities", &center_point, 1_000_000.0, 10)?;
```

### Trajectory Tracking

Track moving objects over time:

```rust
let trajectory = vec![
    (Point::new(40.7128, -74.0060), 1640995200), // Start
    (Point::new(40.7150, -74.0040), 1640995260), // 1 min later
    (Point::new(40.7172, -74.0020), 1640995320), // 2 min later
];

db.insert_trajectory("vehicle:truck001", &trajectory, None)?;

// Query trajectory for a time range
let path = db.query_trajectory("vehicle:truck001", 1640995200, 1640995320)?;
```

## API Overview

Spatio provides a comprehensive set of APIs for spatial and temporal data operations:

### Core Operations
```rust
// Database management
let db = Spatio::memory()?;              // In-memory database
let db = Spatio::open("data.db")?;       // Persistent database
db.sync()?;                                  // Force sync to disk
db.close()?;                                 // Close database

// Basic key-value operations
db.insert("key", b"value", None)?;           // Insert data
let value = db.get("key")?;                  // Get data
db.delete("key")?;                           // Delete data
```

### Spatial Operations
```rust
// Point storage and indexing
db.insert_point("locations:nyc", &point, None)?;
db.insert_point_with_geohash("cities", &point, 8, b"data", None)?;
db.insert_point_with_s2("places", &point, 16, b"data", None)?;

// Spatial queries
let nearby = db.find_nearest_neighbors("cities", &center, 1000.0, 10)?;
let within = db.within("locations", &bounding_box)?;
let intersecting = db.intersects("regions", &area)?;

// Spatial indexing
db.create_spatial_index("locations")?;       // Manual index creation
let stats = db.spatial_stats()?;             // Index statistics
```

### Advanced Geometry
```rust
// Polygon operations
db.insert_polygon("buildings", &polygon, b"metadata", None)?;
let containing = db.geometries_containing_point("buildings", &point)?;
let area = db.total_polygon_area("buildings")?;

// LineString operations
db.insert_linestring("roads", &linestring, b"road_data", None)?;
let distance = db.nearest_geometry_distance("roads", &point)?;

// Geometry queries
let geometries = db.geometries_within_bounds("objects", &min_coord, &max_coord)?;
let intersects = db.intersects_geometry("shapes", &query_geometry)?;
```

### Trajectory Tracking
```rust
// Insert trajectory data
let trajectory = vec![
    (Point::new(40.0, -74.0), timestamp1),
    (Point::new(40.1, -74.1), timestamp2),
];
db.insert_trajectory("vehicle:001", &trajectory, None)?;

// Query trajectory by time range
let path = db.query_trajectory("vehicle:001", start_time, end_time)?;
```

### Atomic Operations
```rust
// Batch operations with atomicity
db.atomic(|batch| {
    batch.insert("key1", b"value1", None)?;
    batch.insert("key2", b"value2", None)?;
    batch.delete("old_key")?;
    Ok(())
})?;
```

### Time-to-Live (TTL)
```rust
use std::time::Duration;

// Insert with TTL
let opts = SetOptions::with_ttl(Duration::from_secs(3600));
db.insert("temp:data", b"expires_in_1hour", Some(opts))?;

// Manual cleanup
db.cleanup_expired()?;
```

## Configuration

```rust
use spatio::{Config, SyncPolicy};

let mut config = Config::default();
config.sync_policy = SyncPolicy::EverySecond;
config.auto_shrink_percentage = 100;
config.max_dimensions = 3; // For 3D spatial data

let db = Spatio::memory()?;
db.set_config(config)?;
```

## Development Status

Spatio is currently in **early development** (v0.1.x). The core functionality is working, but the API may change before v1.0.

### Implemented
- [x] Fast in-memory key-value storage
- [x] Atomic batch operations
- [x] TTL/expiration support
- [x] AOF persistence with replay
- [x] Geographic point storage
- [x] Automatic spatial indexing
- [x] Trajectory tracking and queries
- [x] Nearby point search
- [x] Thread-safe concurrent operations
- [x] Distance calculations
- [x] Comprehensive examples

### Planned
- [ ] Query optimization
- [ ] Advanced persistence features
- [ ] Performance benchmarking tools
- [ ] GeoJSON import/export


## Documentation

- [API Documentation](https://docs.rs/spatio) (Generated from code)
- [Examples](examples/) - Multiple focused examples and comprehensive demos
- [Benchmarks](benches/) - Performance testing suite
- [Assets](assets/) - Logo and branding materials

### Logo Usage

The Spatio logo is available at `assets/images/logo-min.png`. When using the logo in your own projects or documentation, please maintain appropriate attribution.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/pkvartsianyi/spatio.git
cd spatio

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

Spatio is inspired by:
- [BuntDB](https://github.com/tidwall/buntdb) - Fast embeddable in-memory key/value database in Go
- [PostGIS](https://postgis.net/) - For spatial database operations

## Contact

- **Issues**: [GitHub Issues](https://github.com/pkvartsianyi/spatio/issues)
- **Discussions**: [GitHub Discussions](https://github.com/pkvartsianyi/spatio/discussions)

---
