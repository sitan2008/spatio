use spatio_lite::{
    BoundingBox, CoordinateSystem, GeohashUtils, Point, S2Utils, SpatialAnalysis, SpatialKey,
};

#[test]
fn test_point_creation_and_validation() {
    // Test valid coordinates
    let point1 = Point::new(40.7128, -74.0060);
    assert_eq!(point1.lat, 40.7128);
    assert_eq!(point1.lon, -74.0060);

    // Test extreme valid coordinates
    let north_pole = Point::new(90.0, 0.0);
    assert_eq!(north_pole.lat, 90.0);
    assert_eq!(north_pole.lon, 0.0);

    let south_pole = Point::new(-90.0, 0.0);
    assert_eq!(south_pole.lat, -90.0);
    assert_eq!(south_pole.lon, 0.0);

    let date_line_east = Point::new(0.0, 180.0);
    assert_eq!(date_line_east.lat, 0.0);
    assert_eq!(date_line_east.lon, 180.0);

    let date_line_west = Point::new(0.0, -180.0);
    assert_eq!(date_line_west.lat, 0.0);
    assert_eq!(date_line_west.lon, -180.0);
}

#[test]
fn test_point_distance_calculations() {
    let nyc = Point::new(40.7128, -74.0060);
    let london = Point::new(51.5074, -0.1278);
    let tokyo = Point::new(35.6762, 139.6503);

    // Test distance between NYC and London (approximately 5585 km)
    let nyc_london_distance = nyc.distance_to(&london);
    assert!(nyc_london_distance > 5500000.0 && nyc_london_distance < 5600000.0);

    // Test distance between NYC and Tokyo (approximately 10838 km)
    let nyc_tokyo_distance = nyc.distance_to(&tokyo);
    assert!(nyc_tokyo_distance > 10800000.0 && nyc_tokyo_distance < 10900000.0);

    // Test distance to self
    let self_distance = nyc.distance_to(&nyc);
    assert!(self_distance < 1.0); // Should be essentially zero

    // Test very close points
    let close_point = Point::new(40.7129, -74.0059);
    let close_distance = nyc.distance_to(&close_point);
    assert!(close_distance > 0.0 && close_distance < 200.0); // Should be very small
}

#[test]
fn test_point_bearing_calculations() {
    let start = Point::new(40.7128, -74.0060);
    let north = Point::new(41.7128, -74.0060);
    let east = Point::new(40.7128, -73.0060);
    let south = Point::new(39.7128, -74.0060);
    let west = Point::new(40.7128, -75.0060);

    // Test bearing to north (should be close to 0 degrees)
    let bearing_north = start.bearing_to(&north);
    assert!(bearing_north.abs() < 5.0 || bearing_north.abs() > 355.0);

    // Test bearing to east (should be close to 90 degrees)
    let bearing_east = start.bearing_to(&east);
    assert!((bearing_east - 90.0).abs() < 10.0);

    // Test bearing to south (should be close to 180 degrees)
    let bearing_south = start.bearing_to(&south);
    assert!((bearing_south - 180.0).abs() < 5.0);

    // Test bearing to west (should be close to 270 degrees)
    let bearing_west = start.bearing_to(&west);
    assert!((bearing_west - 270.0).abs() < 10.0);
}

#[test]
fn test_geohash_generation_precision() {
    let point = Point::new(40.7128, -74.0060);

    // Test different precision levels
    for precision in 1..=12 {
        let geohash = point.to_geohash(precision).unwrap();
        assert_eq!(geohash.len(), precision);
        assert!(!geohash.is_empty());

        // Verify geohash contains only valid characters
        for ch in geohash.chars() {
            assert!("0123456789bcdefghjkmnpqrstuvwxyz".contains(ch));
        }
    }

    // Test specific precision cases
    let geohash_5 = point.to_geohash(5).unwrap();
    let geohash_8 = point.to_geohash(8).unwrap();
    assert!(geohash_8.starts_with(&geohash_5));
}

#[test]
fn test_s2_cell_generation() {
    let point = Point::new(40.7128, -74.0060);

    // Test different levels
    for level in 1..=20 {
        let cell_id = point.to_s2_cell(level).unwrap();
        assert!(cell_id > 0);
    }

    // Test that higher levels give different (more precise) cell IDs
    let cell_10 = point.to_s2_cell(10).unwrap();
    let cell_15 = point.to_s2_cell(15).unwrap();
    let cell_20 = point.to_s2_cell(20).unwrap();

    // Higher level cells should be different
    assert_ne!(cell_10, cell_15);
    assert_ne!(cell_15, cell_20);
}

#[test]
fn test_bounding_box_operations() {
    let min_point = Point::new(40.0, -75.0);
    let max_point = Point::new(41.0, -74.0);
    let bbox = BoundingBox::new(40.0, -75.0, 41.0, -74.0);

    // Test points inside the bounding box
    let inside_point1 = Point::new(40.5, -74.5);
    let inside_point2 = Point::new(40.1, -74.9);
    let inside_point3 = Point::new(40.9, -74.1);

    assert!(bbox.contains(&inside_point1));
    assert!(bbox.contains(&inside_point2));
    assert!(bbox.contains(&inside_point3));

    // Test points outside the bounding box
    let outside_point1 = Point::new(39.9, -74.5); // South of bbox
    let outside_point2 = Point::new(41.1, -74.5); // North of bbox
    let outside_point3 = Point::new(40.5, -75.1); // West of bbox
    let outside_point4 = Point::new(40.5, -73.9); // East of bbox

    assert!(!bbox.contains(&outside_point1));
    assert!(!bbox.contains(&outside_point2));
    assert!(!bbox.contains(&outside_point3));
    assert!(!bbox.contains(&outside_point4));

    // Test boundary points
    assert!(bbox.contains(&min_point));
    assert!(bbox.contains(&max_point));
}

#[test]
fn test_bounding_box_from_points() {
    let points = vec![
        Point::new(40.0, -75.0),
        Point::new(41.0, -74.0),
        Point::new(40.5, -74.5),
        Point::new(39.8, -75.2), // This should extend the bbox
        Point::new(41.2, -73.8), // This should extend the bbox
    ];

    // Create bbox from all points by finding min/max bounds
    let min_lat = points.iter().map(|p| p.lat).fold(f64::INFINITY, f64::min);
    let max_lat = points
        .iter()
        .map(|p| p.lat)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_lon = points.iter().map(|p| p.lon).fold(f64::INFINITY, f64::min);
    let max_lon = points
        .iter()
        .map(|p| p.lon)
        .fold(f64::NEG_INFINITY, f64::max);
    let bbox = BoundingBox::new(min_lat, min_lon, max_lat, max_lon);

    // Verify all points are contained
    for point in &points {
        assert!(bbox.contains(point));
    }

    // Verify bbox bounds
    assert_eq!(bbox.min.lat, 39.8);
    assert_eq!(bbox.min.lon, -75.2);
    assert_eq!(bbox.max.lat, 41.2);
    assert_eq!(bbox.max.lon, -73.8);
}

#[test]
fn test_spatial_key_generation() {
    // Test geohash keys
    let geohash_key = SpatialKey::geohash("sensor", "dr5regw3");
    assert_eq!(geohash_key, "sensor:gh:dr5regw3");

    // Test S2 keys
    let s2_key = SpatialKey::s2_cell("vehicle", 0x123456789abcdef0);
    assert_eq!(s2_key, "vehicle:s2:123456789abcdef0");

    // Test grid keys
    let grid_key = SpatialKey::grid("weather", 100, 200, 8);
    assert_eq!(grid_key, "weather:grid:8:100:200");

    // Test hierarchical keys
    let hier_key = SpatialKey::hierarchical(
        "iot",
        &[
            "building".to_string(),
            "floor".to_string(),
            "room".to_string(),
        ],
    );
    assert_eq!(hier_key, "iot:hier:building:floor:room");
}

#[test]
fn test_coordinate_system_conversions() {
    let lat = 40.7128;
    let lon = -74.0060;

    // Test WGS84 to Web Mercator conversion
    let point = Point::new(lat, lon);
    let mercator_point = CoordinateSystem::wgs84_to_web_mercator(&point);
    assert!(mercator_point.lat.is_finite());
    assert!(mercator_point.lon.is_finite());

    // Test round-trip conversion
    let wgs84_point = CoordinateSystem::web_mercator_to_wgs84(&mercator_point);
    assert!((lat - wgs84_point.lat).abs() < 0.0001);
    assert!((lon - wgs84_point.lon).abs() < 0.0001);

    // Test grid cell conversion (can be negative for negative coordinates)
    let point = Point::new(lat, lon);
    let (grid_x, grid_y) = CoordinateSystem::to_grid_cell(&point, 3);
    // Grid cells can be negative for negative coordinates (west/south)
    assert!(grid_x != 0 || grid_y != 0); // Just verify it returns something reasonable
}

#[test]
fn test_geohash_utilities() {
    let geohash = "dr5regw3";

    // Test geohash decoding
    let decoded_point = GeohashUtils::decode(geohash).unwrap();
    let lat = decoded_point.lat;
    let lon = decoded_point.lon;
    assert!(lat > 40.0 && lat < 41.0);
    assert!(lon > -75.0 && lon < -74.0);

    // Test neighbor calculation
    let neighbors = GeohashUtils::neighbors(geohash).unwrap();
    assert_eq!(neighbors.len(), 8);
    for neighbor in &neighbors {
        assert_eq!(neighbor.len(), geohash.len());
    }

    // Test parent calculation
    let parent = GeohashUtils::parent(geohash);
    if let Some(parent_hash) = parent {
        assert_eq!(parent_hash.len(), geohash.len() - 1);
        assert!(geohash.starts_with(&parent_hash));
    }

    // Test children calculation
    let children = GeohashUtils::children(geohash);
    assert_eq!(children.len(), 32);
    for child in &children {
        assert_eq!(child.len(), geohash.len() + 1);
        assert!(child.starts_with(geohash));
    }

    // Test bounding box calculation
    let bbox = GeohashUtils::bounding_box(geohash).unwrap();
    assert!(bbox.min.lat <= lat && lat <= bbox.max.lat);
    assert!(bbox.min.lon <= lon && lon <= bbox.max.lon);
}

#[test]
fn test_s2_utilities() {
    let cell_id = 0x123456789abcdef0u64;

    // Test cell bounds
    let bounds = S2Utils::cell_bounds(cell_id).unwrap();
    assert!(bounds.min.lat <= bounds.max.lat);
    assert!(bounds.min.lon <= bounds.max.lon);

    // Test neighbors
    let neighbors = S2Utils::neighbors(cell_id);
    assert!(!neighbors.is_empty());
    for neighbor in &neighbors {
        assert_ne!(*neighbor, cell_id);
    }

    // Test parent
    let parent = S2Utils::parent(cell_id);
    if let Some(parent_id) = parent {
        assert_ne!(parent_id, cell_id);
    }

    // Test children
    let children = S2Utils::children(cell_id);
    assert_eq!(children.len(), 4);
    for child in &children {
        assert_ne!(*child, cell_id);
    }
}

#[test]
fn test_spatial_analysis_functions() {
    let center = Point::new(40.7128, -74.0060);
    let points = vec![
        Point::new(40.7130, -74.0058), // Very close
        Point::new(40.7140, -74.0050), // Close
        Point::new(40.7200, -74.0000), // Medium distance
        Point::new(40.8000, -73.9000), // Far
        Point::new(39.0000, -75.0000), // Very far
    ];

    // Test points within radius
    let within_500m = SpatialAnalysis::points_within_radius(&center, 500.0, &points);
    let within_5km = SpatialAnalysis::points_within_radius(&center, 5000.0, &points);
    let within_50km = SpatialAnalysis::points_within_radius(&center, 50000.0, &points);

    assert!(within_500m.len() <= within_5km.len());
    assert!(within_5km.len() <= within_50km.len());

    // Test centroid calculation
    let centroid = SpatialAnalysis::centroid(&points).unwrap();
    assert!(centroid.lat > 39.0 && centroid.lat < 41.0);
    assert!(centroid.lon > -76.0 && centroid.lon < -73.0);

    // Test buffer calculation
    let buffer_points = SpatialAnalysis::buffer_point(&center, 1000.0, 8);
    assert_eq!(buffer_points.len(), 8);

    // All buffer points should be approximately 1000m from center
    for point in &buffer_points {
        let distance = center.distance_to(point);
        assert!((distance - 1000.0).abs() < 50.0); // Should be very accurate now
    }
}

#[test]
fn test_point_in_polygon() {
    // Define a simple square polygon
    let polygon = vec![
        Point::new(40.0, -75.0), // Bottom-left
        Point::new(41.0, -75.0), // Top-left
        Point::new(41.0, -74.0), // Top-right
        Point::new(40.0, -74.0), // Bottom-right
        Point::new(40.0, -75.0), // Close the polygon
    ];

    // Test points inside the polygon
    let inside_point1 = Point::new(40.5, -74.5);
    let inside_point2 = Point::new(40.2, -74.8);
    let inside_point3 = Point::new(40.8, -74.2);

    assert!(SpatialAnalysis::point_in_polygon(&inside_point1, &polygon));
    assert!(SpatialAnalysis::point_in_polygon(&inside_point2, &polygon));
    assert!(SpatialAnalysis::point_in_polygon(&inside_point3, &polygon));

    // Test points outside the polygon
    let outside_point1 = Point::new(39.9, -74.5); // South of polygon
    let outside_point2 = Point::new(41.1, -74.5); // North of polygon
    let outside_point3 = Point::new(40.5, -75.1); // West of polygon
    let outside_point4 = Point::new(40.5, -73.9); // East of polygon

    assert!(!SpatialAnalysis::point_in_polygon(
        &outside_point1,
        &polygon
    ));
    assert!(!SpatialAnalysis::point_in_polygon(
        &outside_point2,
        &polygon
    ));
    assert!(!SpatialAnalysis::point_in_polygon(
        &outside_point3,
        &polygon
    ));
    assert!(!SpatialAnalysis::point_in_polygon(
        &outside_point4,
        &polygon
    ));

    // Test boundary points (behavior may vary based on implementation)
    let boundary_point = Point::new(40.0, -74.5); // On bottom edge
    let result = SpatialAnalysis::point_in_polygon(&boundary_point, &polygon);
    // Don't assert specific result as boundary behavior can vary
    println!("Boundary point result: {}", result);
}

#[test]
fn test_cross_date_line_calculations() {
    // Test points near the international date line
    let point_west = Point::new(0.0, 179.0);
    let point_east = Point::new(0.0, -179.0);

    // Distance across date line should be reasonable
    let distance = point_west.distance_to(&point_east);
    assert!(distance > 0.0 && distance < 300000.0); // Should be roughly 222 km

    // Test geohash generation near date line
    let geohash_west = point_west.to_geohash(8).unwrap();
    let geohash_east = point_east.to_geohash(8).unwrap();
    assert_ne!(geohash_west, geohash_east);
}

#[test]
fn test_polar_region_calculations() {
    // Test points near the poles
    let near_north_pole = Point::new(89.5, 0.0);
    let near_south_pole = Point::new(-89.5, 0.0);

    // Test geohash generation at poles
    let north_geohash = near_north_pole.to_geohash(8).unwrap();
    let south_geohash = near_south_pole.to_geohash(8).unwrap();
    assert_ne!(north_geohash, south_geohash);

    // Test S2 cell generation at poles
    let north_s2 = near_north_pole.to_s2_cell(15).unwrap();
    let south_s2 = near_south_pole.to_s2_cell(15).unwrap();
    assert_ne!(north_s2, south_s2);

    // Test distance calculations involving poles
    let equator_point = Point::new(0.0, 0.0);
    let north_distance = equator_point.distance_to(&near_north_pole);
    let south_distance = equator_point.distance_to(&near_south_pole);

    // Both should be approximately quarter of Earth's circumference
    assert!(north_distance > 9900000.0 && north_distance < 10100000.0);
    assert!(south_distance > 9900000.0 && south_distance < 10100000.0);
}

#[test]
fn test_precision_and_accuracy() {
    let point = Point::new(40.712345678, -74.006012345);

    // Test that high precision coordinates are preserved
    assert!((point.lat - 40.712345678).abs() < 1e-9);
    assert!((point.lon - -74.006012345).abs() < 1e-9);

    // Test geohash precision consistency
    for precision in 1..=12 {
        let geohash = point.to_geohash(precision).unwrap();
        let decoded = GeohashUtils::decode(&geohash).unwrap();

        // Decoded coordinates should be within reasonable bounds based on precision
        let expected_error = match precision {
            1 => 25.0,
            2 => 5.0,
            3 => 1.0,
            4 => 0.2,
            5 => 0.04,
            6 => 0.008,
            7 => 0.002,
            8 => 0.0004,
            _ => 0.0001,
        };

        assert!((decoded.lat - point.lat).abs() < expected_error);
        assert!((decoded.lon - point.lon).abs() < expected_error);
    }
}

#[test]
fn test_edge_case_inputs() {
    // Test with exactly zero coordinates
    let origin = Point::new(0.0, 0.0);
    assert_eq!(origin.lat, 0.0);
    assert_eq!(origin.lon, 0.0);

    let origin_geohash = origin.to_geohash(8).unwrap();
    assert!(!origin_geohash.is_empty());

    // Test with very small coordinate differences
    let point1 = Point::new(40.7128000, -74.0060000);
    let point2 = Point::new(40.7128001, -74.0060001);

    let tiny_distance = point1.distance_to(&point2);
    assert!(tiny_distance > 0.0 && tiny_distance < 1.0);

    // Test bearing calculation with very close points
    let bearing = point1.bearing_to(&point2);
    assert!((0.0..360.0).contains(&bearing));
}

#[test]
fn test_spatial_consistency() {
    let point = Point::new(40.7128, -74.0060);

    // Test that multiple calls return consistent results
    let geohash1 = point.to_geohash(8).unwrap();
    let geohash2 = point.to_geohash(8).unwrap();
    assert_eq!(geohash1, geohash2);

    let s2_cell1 = point.to_s2_cell(15).unwrap();
    let s2_cell2 = point.to_s2_cell(15).unwrap();
    assert_eq!(s2_cell1, s2_cell2);

    // Test distance consistency (should be commutative)
    let other_point = Point::new(51.5074, -0.1278);
    let distance1 = point.distance_to(&other_point);
    let distance2 = other_point.distance_to(&point);
    assert!((distance1 - distance2).abs() < 1.0);
}
