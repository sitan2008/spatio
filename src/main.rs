use spatio_lite::{BoundingBox, Point, SetOptions, SpatialQueryType, SpatioLite};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SpatioLite Spatial Demo");

    // Create an in-memory database
    let db = SpatioLite::memory()?;

    // Spatial point operations
    println!("\n🌍 Spatial Point Operations:");

    // Create points for major cities
    let nyc = Point::new(40.7128, -74.0060);
    let london = Point::new(51.5074, -0.1278);
    let tokyo = Point::new(35.6762, 139.6503);

    // Insert points with geohash indexing
    db.insert_point_with_geohash("cities", &nyc, 10, b"New York City", None)?;
    db.insert_point_with_geohash("cities", &london, 10, b"London", None)?;
    db.insert_point_with_geohash("cities", &tokyo, 10, b"Tokyo", None)?;

    println!("Inserted cities with spatial indexing");

    // Demonstrate geohash generation
    println!("\n🗺️  Geohash Examples:");
    println!("NYC Geohash (8 chars): {}", nyc.to_geohash(8)?);
    println!("London Geohash (8 chars): {}", london.to_geohash(8)?);
    println!("Tokyo Geohash (8 chars): {}", tokyo.to_geohash(8)?);

    // Distance calculations
    println!("\n📏 Distance Calculations:");
    let nyc_to_london_km = nyc.distance_to(&london) / 1000.0;
    let nyc_to_tokyo_km = nyc.distance_to(&tokyo) / 1000.0;
    println!("NYC to London: {:.0} km", nyc_to_london_km);
    println!("NYC to Tokyo: {:.0} km", nyc_to_tokyo_km);

    // S2 cell indexing
    println!("\n🌐 S2 Cell Indexing:");
    db.insert_point_with_s2("poi", &nyc, 16, b"Times Square", None)?;
    let s2_cell = nyc.to_s2_cell(16)?;
    println!("NYC S2 Cell ID (level 16): {:016x}", s2_cell);

    // Multi-index spatial insertion
    println!("\n🔗 Multi-Index Spatial Operations:");
    let central_park = Point::new(40.7851, -73.9683);
    db.insert_geo_multi_index("landmark:001", &central_park, b"Central Park", None)?;
    println!("Inserted Central Park with multiple spatial indexes");

    // UAV tracking demonstration
    println!("\n🚁 UAV Trajectory Tracking:");
    let trajectory = vec![
        (Point::new(40.7128, -74.0060), 1640995200), // Starting point
        (Point::new(40.7150, -74.0040), 1640995230), // 30 seconds later
        (Point::new(40.7172, -74.0020), 1640995260), // 1 minute later
        (Point::new(40.7194, -74.0000), 1640995290), // 1.5 minutes later
    ];

    db.insert_trajectory("uav:alpha", &trajectory, None)?;
    println!(
        "Inserted UAV trajectory with {} waypoints",
        trajectory.len()
    );

    // Query trajectory
    let queried_path = db.query_trajectory("uav:alpha", 1640995200, 1640995290)?;
    println!("Retrieved {} trajectory points", queried_path.len());

    // Spatial batch operations
    println!("\n📦 Spatial Batch Operations:");
    let sensor_points = vec![
        (
            Point::new(40.7500, -73.9800),
            b"Temperature Sensor A".to_vec(),
        ),
        (Point::new(40.7520, -73.9820), b"Humidity Sensor B".to_vec()),
        (
            Point::new(40.7480, -73.9780),
            b"Air Quality Sensor C".to_vec(),
        ),
    ];

    db.insert_spatial_batch(
        &sensor_points,
        |point, data| {
            let sensor_type = String::from_utf8_lossy(data);
            format!(
                "sensor:{}:{}_{}",
                sensor_type.split_whitespace().last().unwrap_or("unknown"),
                point.lat,
                point.lon
            )
        },
        Some(SetOptions::with_ttl(Duration::from_secs(3600))), // 1 hour TTL
    )?;
    println!("Inserted {} sensors with TTL", sensor_points.len());

    // Bounding box query
    println!("\n📍 Bounding Box Query:");
    let manhattan_bbox = BoundingBox::new(40.7000, -74.0200, 40.8000, -73.9000);
    let points_in_manhattan = db.query_bounding_box("cities", &manhattan_bbox, 8)?;
    println!(
        "Found {} points in Manhattan area",
        points_in_manhattan.len()
    );

    // Nearest neighbor search
    println!("\n🎯 Nearest Neighbor Search:");
    let empire_state = Point::new(40.7484, -73.9857);
    let nearby_cities = db.find_nearest_neighbors("cities", &empire_state, 10000.0, 5)?; // 10km radius
    println!(
        "Found {} cities within 10km of Empire State Building",
        nearby_cities.len()
    );
    for (key, _value, _point, distance) in nearby_cities {
        println!("  {} at distance: {:.0}m", key, distance);
    }

    // Adaptive spatial queries
    println!("\n🧠 Adaptive Spatial Queries:");
    let query = SpatialQueryType::Point {
        point: central_park,
        radius_meters: 5000.0,
    };
    let adaptive_results = db.query_spatial_adaptive("landmark", query)?;
    println!("Adaptive query found {} results", adaptive_results.len());

    // Show spatial statistics
    println!("\n📈 Spatial Database Statistics:");
    let spatial_stats = db.spatial_stats()?;
    println!("Total spatial points: {}", spatial_stats.total_points);
    println!("Geohash indexes: {:?}", spatial_stats.geohash_indexes);
    println!("S2 cell indexes: {:?}", spatial_stats.s2_indexes);
    println!("Grid indexes: {}", spatial_stats.grid_indexes);

    // General database stats
    let db_stats = db.stats()?;
    println!("Total keys in database: {}", db_stats.key_count);

    println!("\n✅ Spatial demo completed successfully!");
    println!("🌍 SpatioLite demonstrated: points, trajectories, spatial indexing, and queries!");

    Ok(())
}
