//! Debug test for spatial index integration

use spatio::{Point, Spatio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing spatial index integration...");

    let db = Spatio::memory()?;

    // Insert some test points
    println!("📍 Inserting test points...");
    let nyc = Point::new(40.7128, -74.0060);
    let boston = Point::new(42.3601, -71.0589);
    let philadelphia = Point::new(39.9526, -75.1652);

    db.insert_point("cities:nyc", &nyc, None)?;
    db.insert_point("cities:boston", &boston, None)?;
    db.insert_point("cities:philadelphia", &philadelphia, None)?;

    // Also insert with geohash
    db.insert_point_with_geohash("spatial_grid", &nyc, 8, b"New York", None)?;
    db.insert_point_with_geohash("spatial_grid", &boston, 8, b"Boston", None)?;
    db.insert_point_with_geohash("spatial_grid", &philadelphia, 8, b"Philadelphia", None)?;

    println!("✅ Points inserted");

    // Test nearest neighbor search before spatial index
    println!("\n🔍 Testing without explicit spatial index...");
    let center = Point::new(40.0, -74.0);
    let results_before = db.find_nearest_neighbors("cities", &center, 500000.0, 10)?;
    println!(
        "Found {} results before spatial index",
        results_before.len()
    );
    for (key, _value, point, distance) in &results_before {
        println!(
            "  {} at ({}, {}) - {:.0}m away",
            key, point.lat, point.lon, distance
        );
    }

    let grid_results_before = db.find_nearest_neighbors("spatial_grid", &center, 500000.0, 10)?;
    println!(
        "Found {} grid results before spatial index",
        grid_results_before.len()
    );
    for (key, _value, point, distance) in &grid_results_before {
        println!(
            "  {} at ({}, {}) - {:.0}m away",
            key, point.lat, point.lon, distance
        );
    }

    // Manually create spatial index
    println!("\n🔧 Creating spatial index...");
    match db.create_spatial_index("cities") {
        Ok(()) => println!("✅ Spatial index created for 'cities'"),
        Err(e) => println!("❌ Failed to create spatial index: {}", e),
    }

    match db.create_spatial_index("spatial_grid") {
        Ok(()) => println!("✅ Spatial index created for 'spatial_grid'"),
        Err(e) => println!("❌ Failed to create spatial index: {}", e),
    }

    // Test nearest neighbor search after spatial index
    println!("\n🚀 Testing with spatial index...");
    let results_after = db.find_nearest_neighbors("cities", &center, 500000.0, 10)?;
    println!("Found {} results after spatial index", results_after.len());
    for (key, _value, point, distance) in &results_after {
        println!(
            "  {} at ({}, {}) - {:.0}m away",
            key, point.lat, point.lon, distance
        );
    }

    let grid_results_after = db.find_nearest_neighbors("spatial_grid", &center, 500000.0, 10)?;
    println!(
        "Found {} grid results after spatial index",
        grid_results_after.len()
    );
    for (key, _value, point, distance) in &grid_results_after {
        println!(
            "  {} at ({}, {}) - {:.0}m away",
            key, point.lat, point.lon, distance
        );
    }

    // Test automatic spatial index creation
    println!("\n🤖 Testing automatic spatial index creation...");
    let auto_results = db.find_nearest_neighbors("new_prefix", &center, 500000.0, 10)?;
    println!(
        "Found {} results with auto index creation",
        auto_results.len()
    );

    // Add some points to the new prefix and test again
    db.insert_point("new_prefix:test1", &Point::new(40.5, -74.5), None)?;
    db.insert_point("new_prefix:test2", &Point::new(41.0, -73.5), None)?;

    let auto_results_2 = db.find_nearest_neighbors("new_prefix", &center, 500000.0, 10)?;
    println!("Found {} results after adding points", auto_results_2.len());
    for (key, _value, point, distance) in &auto_results_2 {
        println!(
            "  {} at ({}, {}) - {:.0}m away",
            key, point.lat, point.lon, distance
        );
    }

    println!("\n🎉 Spatial index debug test completed!");
    Ok(())
}
