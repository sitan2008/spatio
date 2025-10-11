use spatio_lite::{BoundingBox, Point, SpatioLite};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpatioLite - Spatial Queries Example");
    println!("====================================");

    // Create an in-memory database
    let db = SpatioLite::memory()?;
    println!("✓ Created in-memory database");

    // Sample data: Major world cities with geohash indexing
    let cities = vec![
        ("New York", Point::new(40.7128, -74.0060)),
        ("London", Point::new(51.5074, -0.1278)),
        ("Tokyo", Point::new(35.6762, 139.6503)),
        ("Paris", Point::new(48.8566, 2.3522)),
        ("Sydney", Point::new(-33.8688, 151.2093)),
        ("Los Angeles", Point::new(34.0522, -118.2437)),
        ("Berlin", Point::new(52.5200, 13.4050)),
        ("Moscow", Point::new(55.7558, 37.6176)),
        ("Cairo", Point::new(30.0444, 31.2357)),
        ("Mumbai", Point::new(19.0760, 72.8777)),
    ];

    // Insert cities with spatial indexing
    for (name, point) in &cities {
        db.insert_point_with_geohash("world_cities", point, 8, name.as_bytes(), None)?;
    }
    println!("✓ Inserted {} cities with spatial indexing", cities.len());

    // === DISTANCE-BASED QUERIES ===
    println!("\n--- Distance-Based Queries ---");

    let reference_point = Point::new(48.8566, 2.3522); // Paris
    println!("Reference point: Paris (48.8566°N, 2.3522°E)");

    // Find cities within 1000km of Paris
    let nearby_1000km =
        db.find_nearest_neighbors("world_cities", &reference_point, 1_000_000.0, 10)?;
    println!("\nCities within 1000km of Paris:");
    for (city_key, _value, point, distance) in &nearby_1000km {
        println!("  {} - {:.0} km", city_key, distance / 1000.0);
    }

    // Find cities within 2000km of Paris
    let nearby_2000km =
        db.find_nearest_neighbors("world_cities", &reference_point, 2_000_000.0, 10)?;
    println!(
        "\nCities within 2000km of Paris: {} cities",
        nearby_2000km.len()
    );

    // Find the 3 nearest cities to Paris
    let nearest_3 =
        db.find_nearest_neighbors("world_cities", &reference_point, f64::INFINITY, 3)?;
    println!("\n3 nearest cities to Paris:");
    for (city_key, _value, _point, distance) in &nearest_3 {
        println!("  {} - {:.0} km", city_key, distance / 1000.0);
    }

    // === BOUNDING BOX QUERIES ===
    println!("\n--- Bounding Box Queries ---");

    // Define Europe bounding box (approximately)
    let europe_bbox = BoundingBox::new(35.0, -10.0, 70.0, 40.0);
    println!("Europe bounding box: 35°N-70°N, 10°W-40°E");

    let european_cities = db.within("world_cities", &europe_bbox)?;
    println!("\nEuropean cities:");
    for (city_key, _value, point) in &european_cities {
        println!("  {} at ({:.4}°N, {:.4}°E)", city_key, point.lat, point.lon);
    }

    // Define Asia-Pacific bounding box
    let asia_pacific_bbox = BoundingBox::new(-45.0, 100.0, 60.0, 180.0);
    println!("\nAsia-Pacific bounding box: 45°S-60°N, 100°E-180°E");

    let asia_pacific_cities = db.within("world_cities", &asia_pacific_bbox)?;
    println!("\nAsia-Pacific cities:");
    for (city_key, _value, point) in &asia_pacific_cities {
        println!("  {} at ({:.4}°N, {:.4}°E)", city_key, point.lat, point.lon);
    }

    // === GEOHASH-BASED QUERIES ===
    println!("\n--- Geohash Analysis ---");

    // Show geohash values for cities
    println!("City geohashes (8 characters):");
    for (name, point) in &cities {
        let geohash = point.to_geohash(8)?;
        println!("  {}: {}", name, geohash);
    }

    // Group cities by geohash prefix (first 3 characters)
    let mut geohash_groups: std::collections::HashMap<String, Vec<&str>> =
        std::collections::HashMap::new();
    for (name, point) in &cities {
        let geohash = point.to_geohash(3)?;
        geohash_groups
            .entry(geohash)
            .or_insert_with(Vec::new)
            .push(name);
    }

    println!("\nCities grouped by geohash prefix (3 chars):");
    for (geohash, city_list) in &geohash_groups {
        println!("  {}: {:?}", geohash, city_list);
    }

    // === SPATIAL INTERSECTIONS ===
    println!("\n--- Spatial Intersections ---");

    // Create some points of interest
    let landmarks = vec![
        ("Eiffel Tower", Point::new(48.8584, 2.2945)),
        ("Statue of Liberty", Point::new(40.6892, -74.0445)),
        ("Big Ben", Point::new(51.5007, -0.1246)),
        ("Sydney Opera House", Point::new(-33.8568, 151.2153)),
    ];

    for (name, point) in &landmarks {
        db.insert_point_with_geohash("landmarks", point, 10, name.as_bytes(), None)?;
    }
    println!("✓ Added {} landmarks", landmarks.len());

    // Find intersections (landmarks near cities)
    for (city_name, city_point) in &cities {
        let nearby_landmarks = db.find_nearest_neighbors("landmarks", city_point, 50_000.0, 5)?; // 50km radius
        if !nearby_landmarks.is_empty() {
            println!("\nLandmarks near {}:", city_name);
            for (landmark_key, _value, _point, distance) in &nearby_landmarks {
                println!("  {} - {:.1} km away", landmark_key, distance / 1000.0);
            }
        }
    }

    // === ADVANCED SPATIAL QUERIES ===
    println!("\n--- Advanced Spatial Analysis ---");

    // Calculate center point of all cities
    let total_lat: f64 = cities.iter().map(|(_, p)| p.lat).sum();
    let total_lon: f64 = cities.iter().map(|(_, p)| p.lon).sum();
    let center = Point::new(
        total_lat / cities.len() as f64,
        total_lon / cities.len() as f64,
    );

    println!(
        "Geographic center of all cities: ({:.4}°N, {:.4}°E)",
        center.lat, center.lon
    );

    // Find the city closest to the center
    let center_nearest = db.find_nearest_neighbors("world_cities", &center, f64::INFINITY, 1)?;
    if let Some((closest_city, _value, _point, distance)) = center_nearest.first() {
        println!(
            "City closest to center: {} ({:.0} km away)",
            closest_city,
            distance / 1000.0
        );
    }

    // Calculate maximum distance between any two cities
    let mut max_distance = 0.0;
    let mut farthest_pair = ("", "");

    for (i, (name1, point1)) in cities.iter().enumerate() {
        for (name2, point2) in cities.iter().skip(i + 1) {
            let distance = point1.distance_to(point2);
            if distance > max_distance {
                max_distance = distance;
                farthest_pair = (name1, name2);
            }
        }
    }

    println!(
        "Farthest city pair: {} ↔ {} ({:.0} km)",
        farthest_pair.0,
        farthest_pair.1,
        max_distance / 1000.0
    );

    // === QUERY PERFORMANCE ===
    println!("\n--- Query Performance ---");

    let start = std::time::Instant::now();
    let _all_within_global = db.within(
        "world_cities",
        &BoundingBox::new(-90.0, -180.0, 90.0, 180.0),
    )?;
    let global_query_time = start.elapsed();

    let start = std::time::Instant::now();
    let _nearest_to_paris =
        db.find_nearest_neighbors("world_cities", &reference_point, f64::INFINITY, 10)?;
    let nearest_query_time = start.elapsed();

    println!("Global bounding box query: {:?}", global_query_time);
    println!("Nearest neighbors query: {:?}", nearest_query_time);

    // Display final statistics
    let spatial_stats = db.spatial_stats()?;
    println!("\n--- Database Statistics ---");
    println!("Total spatial points: {}", spatial_stats.total_points);
    println!("Geohash indexes: {:?}", spatial_stats.geohash_indexes);
    println!("S2 cell indexes: {:?}", spatial_stats.s2_indexes);

    println!("\n🎉 Spatial queries example completed successfully!");
    println!("\nKey takeaways:");
    println!("- Distance-based queries help find nearby locations");
    println!("- Bounding boxes efficiently filter geographic regions");
    println!("- Geohash indexing enables fast spatial lookups");
    println!("- Multiple indexing strategies can be combined for optimal performance");

    Ok(())
}
