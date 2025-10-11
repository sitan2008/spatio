use spatio_lite::{Point, SetOptions, SpatioLite};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpatioLite - Getting Started Example");
    println!("=====================================");

    // Create an in-memory database
    let db = SpatioLite::memory()?;
    println!("✓ Created in-memory database");

    // Basic key-value operations
    db.insert("hello", b"world", None)?;
    let value = db.get("hello")?.unwrap();
    println!(
        "✓ Basic storage: hello = {}",
        String::from_utf8_lossy(&value)
    );

    // Store data with TTL (time-to-live)
    let ttl_options = SetOptions::with_ttl(Duration::from_secs(5));
    db.insert("temp_data", b"expires_soon", Some(ttl_options))?;
    println!("✓ Stored data with 5-second TTL");

    // Spatial point operations
    let new_york = Point::new(40.7128, -74.0060);
    let london = Point::new(51.5074, -0.1278);

    db.insert_point("cities:nyc", &new_york, None)?;
    db.insert_point("cities:london", &london, None)?;
    println!("✓ Stored geographic points for NYC and London");

    // Calculate distance between cities
    let distance_km = new_york.distance_to(&london) / 1000.0;
    println!("✓ Distance NYC ↔ London: {:.0} km", distance_km);

    // Spatial indexing with geohash
    db.insert_point_with_geohash("indexed_cities", &new_york, 8, b"New York City", None)?;
    db.insert_point_with_geohash("indexed_cities", &london, 8, b"London", None)?;
    println!("✓ Added cities with spatial indexing");

    // Find nearby cities (within 2000km of NYC)
    let nearby = db.find_nearest_neighbors("indexed_cities", &new_york, 2_000_000.0, 5)?;
    println!("✓ Found {} cities within 2000km of NYC", nearby.len());

    // Atomic batch operations
    db.atomic(|batch| {
        batch.insert("sensor:temperature", b"22.5C", None)?;
        batch.insert("sensor:humidity", b"65pct", None)?;
        batch.insert("sensor:pressure", b"1013.25 hPa", None)?;
        Ok(())
    })?;
    println!("✓ Performed atomic batch insert of sensor data");

    // Check database statistics
    let stats = db.stats()?;
    println!("✓ Database contains {} keys", stats.key_count);

    // Wait a moment to see TTL in action
    println!("\nWaiting 6 seconds to demonstrate TTL...");
    std::thread::sleep(Duration::from_secs(6));

    // Try to retrieve expired data
    match db.get("temp_data")? {
        Some(_) => println!("✗ TTL data still exists (unexpected)"),
        None => println!("✓ TTL data expired as expected"),
    }

    println!("\n🎉 Getting started example completed successfully!");
    println!("\nNext steps:");
    println!("- Try the 'spatial_queries' example for advanced spatial operations");
    println!("- Check out 'geometry_demo' for polygon and linestring operations");
    println!("- See 'trajectory_tracking' for time-series spatial data");

    Ok(())
}
