use spatio_lite::{
    BoundingBox, Coordinate, Geometry, GeometryOps, LineString, LinearRing, Point, Polygon,
    SetOptions, SpatioLite,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpatioLite Comprehensive Spatial & Geometry Demo");

    // Create an in-memory database
    let db = SpatioLite::memory()?;

    // === BASIC SPATIAL POINT OPERATIONS ===
    println!("Basic Spatial Point Operations:");

    // Create points for major cities
    let nyc = Point::new(40.7128, -74.0060);
    let london = Point::new(51.5074, -0.1278);
    let tokyo = Point::new(35.6762, 139.6503);

    // Insert points with geohash indexing
    db.insert_point_with_geohash("cities", &nyc, 10, b"New York City", None)?;
    db.insert_point_with_geohash("cities", &london, 10, b"London", None)?;
    db.insert_point_with_geohash("cities", &tokyo, 10, b"Tokyo", None)?;

    println!("Inserted cities with spatial indexing");

    // === ADVANCED GEOMETRY OPERATIONS ===
    println!("Advanced Geometry Operations:");

    // Create a polygon representing Central Park
    let central_park_coords = vec![
        Coordinate::new(-73.9733, 40.7644), // SW corner
        Coordinate::new(-73.9500, 40.7644), // SE corner
        Coordinate::new(-73.9500, 40.7997), // NE corner
        Coordinate::new(-73.9733, 40.7997), // NW corner
        Coordinate::new(-73.9733, 40.7644), // Close the ring
    ];
    let central_park_ring = LinearRing::new(central_park_coords)?;
    let central_park = Polygon::new(central_park_ring);

    // Insert polygon with spatial indexing
    db.insert_polygon("parks", &central_park, b"Central Park, Manhattan", None)?;
    println!("Inserted Central Park polygon");

    // Keep a copy for later WKT demo
    let central_park_copy = central_park.clone();

    // Create a complex polygon with a hole (representing a building with courtyard)
    let building_exterior = vec![
        Coordinate::new(-73.9850, 40.7580),
        Coordinate::new(-73.9820, 40.7580),
        Coordinate::new(-73.9820, 40.7610),
        Coordinate::new(-73.9850, 40.7610),
        Coordinate::new(-73.9850, 40.7580),
    ];
    let building_hole = vec![
        Coordinate::new(-73.9840, 40.7590),
        Coordinate::new(-73.9830, 40.7590),
        Coordinate::new(-73.9830, 40.7600),
        Coordinate::new(-73.9840, 40.7600),
        Coordinate::new(-73.9840, 40.7590),
    ];

    let exterior_ring = LinearRing::new(building_exterior)?;
    let hole_ring = LinearRing::new(building_hole)?;
    let building_with_courtyard = Polygon::with_holes(exterior_ring, vec![hole_ring]);

    db.insert_polygon(
        "buildings",
        &building_with_courtyard,
        b"Office Building with Courtyard",
        None,
    )?;
    println!("Inserted building polygon with courtyard hole");

    // Create linestring geometries (streets, routes)
    let broadway_coords = vec![
        Coordinate::new(-73.9857, 40.7484), // Times Square area
        Coordinate::new(-73.9867, 40.7505),
        Coordinate::new(-73.9877, 40.7526),
        Coordinate::new(-73.9887, 40.7547),
        Coordinate::new(-73.9897, 40.7568), // Columbus Circle area
    ];
    let broadway = LineString::new(broadway_coords)?;

    db.insert_linestring("streets", &broadway, b"Broadway (partial)", None)?;
    println!("Inserted Broadway street linestring");

    // Create a subway route
    let subway_coords = vec![
        Coordinate::new(-73.9857, 40.7589), // 42nd St
        Coordinate::new(-73.9857, 40.7614), // 47th St
        Coordinate::new(-73.9857, 40.7640), // 51st St
        Coordinate::new(-73.9857, 40.7666), // 57th St
    ];
    let subway_line = LineString::new(subway_coords)?;

    db.insert_linestring("transit", &subway_line, b"N/Q/R/W Line (partial)", None)?;
    println!("Inserted subway line");

    // Demonstrate geohash generation
    println!("Geohash Examples:");
    println!("NYC Geohash (8 chars): {}", nyc.to_geohash(8)?);
    println!("London Geohash (8 chars): {}", london.to_geohash(8)?);
    println!("Tokyo Geohash (8 chars): {}", tokyo.to_geohash(8)?);

    // Distance calculations
    println!("Distance Calculations:");
    let nyc_to_london_km = nyc.distance_to(&london) / 1000.0;
    let nyc_to_tokyo_km = nyc.distance_to(&tokyo) / 1000.0;
    println!("NYC to London: {:.0} km", nyc_to_london_km);
    println!("NYC to Tokyo: {:.0} km", nyc_to_tokyo_km);

    // S2 cell indexing
    println!("S2 Cell Indexing:");
    db.insert_point_with_s2("poi", &nyc, 16, b"Times Square", None)?;
    let s2_cell = nyc.to_s2_cell(16)?;
    println!("NYC S2 Cell ID (level 16): {:016x}", s2_cell);

    // Multi-index spatial insertion
    println!("Multi-Index Spatial Operations:");
    let central_park = Point::new(40.7851, -73.9683);
    db.insert_point_with_geohash("landmark", &central_park, 8, b"Central Park", None)?;
    println!("Inserted Central Park with spatial indexing");

    // UAV tracking demonstration
    println!("UAV Trajectory Tracking:");
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
    println!("Spatial Batch Operations:");
    let opts = Some(SetOptions::with_ttl(Duration::from_secs(3600))); // 1 hour TTL
    db.insert_point_with_geohash(
        "sensors",
        &Point::new(40.7500, -73.9800),
        8,
        b"Temperature Sensor A",
        opts.clone(),
    )?;
    db.insert_point_with_geohash(
        "sensors",
        &Point::new(40.7520, -73.9820),
        8,
        b"Humidity Sensor B",
        opts.clone(),
    )?;
    db.insert_point_with_geohash(
        "sensors",
        &Point::new(40.7480, -73.9780),
        8,
        b"Air Quality Sensor C",
        opts,
    )?;
    println!("Inserted 3 sensors with TTL");

    // Bounding box query
    println!("Bounding Box Query:");
    let manhattan_bbox = BoundingBox::new(40.7000, -74.0200, 40.8000, -73.9000);
    let points_in_manhattan = db.within("cities", &manhattan_bbox)?;
    println!(
        "Found {} points in Manhattan area",
        points_in_manhattan.len()
    );

    // Nearest neighbor search
    println!("Nearest Neighbor Search:");
    let empire_state = Point::new(40.7484, -73.9857);
    let nearby_cities = db.find_nearest_neighbors("cities", &empire_state, 10000.0, 5)?; // 10km radius
    println!(
        "Found {} cities within 10km of Empire State Building",
        nearby_cities.len()
    );
    for (key, _value, _point, distance) in nearby_cities {
        println!("  {} at distance: {:.0}m", key, distance);
    }

    // Simple spatial queries
    println!("Simple Spatial Queries:");
    let intersecting_results = db.intersects("landmark", &central_park, 5000.0)?;
    println!(
        "Intersecting query found {} results",
        intersecting_results.len()
    );

    // === GEOMETRY QUERIES AND ANALYSIS ===
    println!("Geometry Queries and Analysis:");

    // Test point-in-polygon queries
    let test_point_in_park = Coordinate::new(-73.9650, 40.7820); // Inside Central Park
    let test_point_outside = Coordinate::new(-73.9400, 40.7500); // Outside park

    let parks_containing_point = db.geometries_containing_point("parks", &test_point_in_park)?;
    println!(
        "Found {} parks containing test point",
        parks_containing_point.len()
    );

    let parks_containing_outside = db.geometries_containing_point("parks", &test_point_outside)?;
    println!(
        "Found {} parks containing outside point",
        parks_containing_outside.len()
    );

    // Bounding box queries
    let bbox_min = Coordinate::new(-74.0000, 40.7500);
    let bbox_max = Coordinate::new(-73.9500, 40.8000);
    let geometries_in_bbox = db.geometries_within_bounds("parks", &bbox_min, &bbox_max)?;
    println!(
        "Found {} geometries in bounding box",
        geometries_in_bbox.len()
    );

    // Calculate areas and lengths
    let total_park_area = db.total_polygon_area("parks")?;
    let total_street_length = db.total_linestring_length("streets")?;
    let total_transit_length = db.total_linestring_length("transit")?;

    println!("Total park area: {:.6} square degrees", total_park_area);
    println!("Total street length: {:.6} degrees", total_street_length);
    println!("Total transit length: {:.6} degrees", total_transit_length);

    // Nearest geometry queries
    let query_point = Coordinate::new(-73.9800, 40.7700);
    if let Some((nearest_key, nearest_geom, distance)) =
        db.nearest_geometry_distance("parks", &query_point)?
    {
        println!("Nearest park: {} at distance {:.6}", nearest_key, distance);
        println!("   Geometry type: {}", nearest_geom.geometry_type());
    }

    // === GEOMETRY UTILITIES AND OPERATIONS ===
    println!("Geometry Utilities:");

    // Create a circular buffer around a point
    let buffer_center = Coordinate::new(-73.9750, 40.7750);
    let buffer_polygon = GeometryOps::buffer_point(&buffer_center, 0.005, 16)?; // ~500m radius

    db.insert_polygon("zones", &buffer_polygon, b"Safety Zone", None)?;
    println!("Created circular buffer zone");

    // Create a rectangular area
    let rect_polygon = GeometryOps::rectangle(-73.9900, 40.7400, -73.9700, 40.7600)?;
    db.insert_polygon("zones", &rect_polygon, b"Commercial District", None)?;
    println!("Created rectangular zone");

    // === GEOMETRY SERIALIZATION AND WKT ===
    println!("Geometry Serialization:");

    // Demonstrate WKT output
    let point_geom = Geometry::Point(Coordinate::new(-73.9857, 40.7484));
    println!("Point WKT: {}", point_geom.to_wkt());

    let line_geom = Geometry::LineString(broadway);
    println!("LineString WKT: {}", line_geom.to_wkt());

    let poly_geom = Geometry::Polygon(central_park_copy);
    println!("Polygon WKT: {}", poly_geom.to_wkt());

    // Test serialization round-trip
    db.insert_geometry("test:serialization", &point_geom, None)?;
    let retrieved_geom = db.get_geometry("test:serialization")?.unwrap();
    println!(
        "Geometry serialization round-trip successful: {}",
        point_geom == retrieved_geom
    );

    // === LIST ALL GEOMETRIES ===
    println!("Geometry Inventory:");

    let all_parks = db.list_geometries("parks")?;
    println!("Parks: {} geometries", all_parks.len());
    for (key, geometry, value) in &all_parks {
        let value_str = value
            .as_ref()
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_else(|| "No description".to_string());
        println!("  {} ({}): {}", key, geometry.geometry_type(), value_str);
    }

    let all_buildings = db.list_geometries("buildings")?;
    println!("Buildings: {} geometries", all_buildings.len());
    for (key, geometry, value) in &all_buildings {
        let value_str = value
            .as_ref()
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_else(|| "No description".to_string());
        println!(
            "  {} ({}): {} - Area: {:.8}",
            key,
            geometry.geometry_type(),
            value_str,
            geometry.area()
        );
    }

    let all_streets = db.list_geometries("streets")?;
    println!("Streets: {} geometries", all_streets.len());
    for (key, geometry, value) in &all_streets {
        let value_str = value
            .as_ref()
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_else(|| "No description".to_string());
        println!(
            "  {} ({}): {} - Length: {:.8}",
            key,
            geometry.geometry_type(),
            value_str,
            geometry.length()
        );
    }

    // Show spatial statistics
    println!("Spatial Database Statistics:");
    let spatial_stats = db.spatial_stats()?;
    println!("Total spatial points: {}", spatial_stats.total_points);
    println!("Geohash indexes: {:?}", spatial_stats.geohash_indexes);
    println!("S2 cell indexes: {:?}", spatial_stats.s2_indexes);
    println!("Grid indexes: {}", spatial_stats.grid_indexes);

    // General database stats
    let db_stats = db.stats()?;
    println!("Total keys in database: {}", db_stats.key_count);

    println!("Comprehensive geometry demo completed successfully!");
    println!("SpatioLite demonstrated:");
    println!("Points, trajectories, and spatial indexing");
    println!("Polygons with holes and complex shapes");
    println!("LineStrings for routes and paths");
    println!("Spatial queries (contains, intersects, within bounds)");
    println!("Geometry operations (buffer, distance, areas)");
    println!("WKT serialization and data persistence");
    println!("Advanced geometry support and analysis");

    Ok(())
}
