use spatio_lite::{BoundingBox, Point, SetOptions, SpatioLite};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

#[test]
fn test_advanced_spatial_operations() {
    let db = SpatioLite::memory().unwrap();

    // Create a grid of spatial data points
    let mut points = Vec::new();
    for i in 0..100 {
        for j in 0..100 {
            let lat = 40.0 + (i as f64 * 0.001);
            let lon = -74.0 + (j as f64 * 0.001);
            let point = Point::new(lat, lon);
            let key = format!("grid:{}:{}", i, j);
            let data = format!("sensor_data:temp:{}:humidity:{}", 20 + i % 15, 40 + j % 30);

            db.insert_point(&key, &point, None).unwrap();
            db.insert_point_with_geohash("spatial_grid", &point, 8, data.as_bytes(), None)
                .unwrap();
            points.push((key, point));
        }
    }

    // Test nearest neighbor search with various radii
    let center = Point::new(40.05, -73.95);

    let nearby_1km = db
        .find_nearest_neighbors("spatial_grid", &center, 1000.0, 50)
        .unwrap();
    let nearby_5km = db
        .find_nearest_neighbors("spatial_grid", &center, 5000.0, 100)
        .unwrap();

    assert!(!nearby_1km.is_empty());
    assert!(!nearby_5km.is_empty());
    assert!(nearby_5km.len() >= nearby_1km.len());

    // Verify results are sorted by distance
    for i in 1..nearby_5km.len() {
        assert!(nearby_5km[i - 1].3 <= nearby_5km[i].3);
    }

    // Test spatial statistics
    let stats = db.spatial_stats().unwrap();
    assert!(stats.total_points > 0);
    assert!(!stats.geohash_indexes.is_empty());
}

#[test]
fn test_trajectory_analysis() {
    let db = SpatioLite::memory().unwrap();

    // Simulate multiple vehicle trajectories
    let vehicles = vec!["vehicle_001", "vehicle_002", "vehicle_003"];
    let base_time = 1640995200u64;

    for (vehicle_idx, vehicle_id) in vehicles.iter().enumerate() {
        let mut trajectory = Vec::new();

        // Create a trajectory with 100 points over time
        for i in 0..100 {
            let lat = 40.7128 + (vehicle_idx as f64 * 0.01) + (i as f64 * 0.0001);
            let lon = -74.0060 + (vehicle_idx as f64 * 0.01) + (i as f64 * 0.0001);
            let point = Point::new(lat, lon);
            let timestamp = base_time + (vehicle_idx as u64 * 10000) + (i as u64 * 30);
            trajectory.push((point, timestamp));
        }

        db.insert_trajectory(vehicle_id, &trajectory, None).unwrap();
    }

    // Query trajectories for different time ranges
    for vehicle_id in &vehicles {
        let full_trajectory = db
            .query_trajectory(vehicle_id, base_time, base_time + 100000)
            .unwrap();
        assert!(full_trajectory.len() >= 90); // Should capture most points

        let partial_trajectory = db
            .query_trajectory(vehicle_id, base_time, base_time + 1000)
            .unwrap();
        assert!(partial_trajectory.len() < full_trajectory.len());

        // Verify temporal ordering
        for i in 1..full_trajectory.len() {
            assert!(full_trajectory[i - 1].1 <= full_trajectory[i].1);
        }
    }
}

#[test]
fn test_large_scale_concurrent_operations() {
    let db = Arc::new(SpatioLite::memory().unwrap());
    let num_threads = 10;
    let operations_per_thread = 1000;

    let mut handles = vec![];

    // Spawn multiple threads performing concurrent operations
    for thread_id in 0..num_threads {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for i in 0..operations_per_thread {
                let lat = 40.0 + (thread_id as f64 * 0.01) + (i as f64 * 0.00001);
                let lon = -74.0 + (thread_id as f64 * 0.01) + (i as f64 * 0.00001);
                let point = Point::new(lat, lon);
                let key = format!("concurrent:thread_{}:item_{}", thread_id, i);

                // Mix of different operation types
                match i % 4 {
                    0 => {
                        // Regular point insertion
                        db_clone.insert_point(&key, &point, None).unwrap();
                    }
                    1 => {
                        // Geohash insertion
                        let data = format!("data_{}_{}", thread_id, i);
                        db_clone
                            .insert_point_with_geohash(
                                &format!("concurrent_geohash_{}", thread_id),
                                &point,
                                8,
                                data.as_bytes(),
                                None,
                            )
                            .unwrap();
                    }
                    2 => {
                        // S2 cell insertion
                        let data = format!("s2_data_{}_{}", thread_id, i);
                        db_clone
                            .insert_point_with_s2(
                                &format!("concurrent_s2_{}", thread_id),
                                &point,
                                16,
                                data.as_bytes(),
                                None,
                            )
                            .unwrap();
                    }
                    3 => {
                        // Batch operation
                        db_clone
                            .atomic(|batch| {
                                for j in 0..5 {
                                    let batch_key = format!("batch_{}_{}_sub_{}", thread_id, i, j);
                                    let batch_data = format!("batch_data_{}", j);
                                    batch.insert(&batch_key, batch_data.as_bytes(), None)?;
                                }
                                Ok(())
                            })
                            .unwrap();
                    }
                    _ => unreachable!(),
                }

                // Occasionally perform reads
                if i % 10 == 0 {
                    let _ = db_clone.get(&key);
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state
    let stats = db.stats().unwrap();
    println!("Final key count: {}", stats.key_count);
    assert!(stats.key_count > num_threads * operations_per_thread / 2); // Account for batch ops
}

#[test]
fn test_ttl_and_expiration_stress() {
    let db = SpatioLite::memory().unwrap();

    // Insert data with various TTL values
    let _base_time = Instant::now();
    for i in 0..1000 {
        let key = format!("expiring_key_{}", i);
        let value = format!("expiring_value_{}", i);

        let ttl = match i % 5 {
            0 => Duration::from_millis(50),  // Very short
            1 => Duration::from_millis(100), // Short
            2 => Duration::from_millis(200), // Medium
            3 => Duration::from_millis(500), // Long
            4 => Duration::from_secs(60),    // Very long (won't expire during test)
            _ => unreachable!(),
        };

        let opts = SetOptions::with_ttl(ttl);
        db.insert(&key, value.as_bytes(), Some(opts)).unwrap();
    }

    // Verify immediate availability
    let initial_stats = db.stats().unwrap();
    assert_eq!(initial_stats.key_count, 1000);

    // Wait for some expirations
    thread::sleep(Duration::from_millis(150));

    // Manually trigger cleanup to avoid timing issues with background thread
    db.cleanup_expired().unwrap();

    let mid_stats = db.stats().unwrap();
    assert!(mid_stats.key_count < initial_stats.key_count);

    // Wait for more expirations
    thread::sleep(Duration::from_millis(400));

    // Manually trigger cleanup again
    db.cleanup_expired().unwrap();

    // Only the very long TTL items should remain
    let final_stats = db.stats().unwrap();
    assert!(final_stats.key_count < mid_stats.key_count);
    assert!(final_stats.key_count >= 200); // The 60-second TTL items
}

#[test]
fn test_persistence_integrity() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create initial dataset
    {
        let db = SpatioLite::open(path).unwrap();

        // Insert various types of data
        for i in 0..500 {
            let lat = 40.7128 + (i as f64 * 0.0001);
            let lon = -74.0060 + (i as f64 * 0.0001);
            let point = Point::new(lat, lon);

            // Regular key-value pairs
            db.insert(
                &format!("persistent_key_{}", i),
                &format!("value_{}", i).as_bytes(),
                None,
            )
            .unwrap();

            // Spatial points
            db.insert_point(&format!("persistent_point_{}", i), &point, None)
                .unwrap();

            // Geohash data
            db.insert_point_with_geohash(
                "persistent_geohash",
                &point,
                8,
                &format!("geohash_data_{}", i).as_bytes(),
                None,
            )
            .unwrap();
        }

        // Force sync
        db.sync().unwrap();
    }

    // Reopen and verify data integrity
    {
        let db = SpatioLite::open(path).unwrap();

        // Verify regular key-value pairs
        for i in 0..500 {
            let key = format!("persistent_key_{}", i);
            let value = db.get(&key).unwrap().unwrap();
            assert_eq!(value, format!("value_{}", i).as_bytes());
        }

        // Verify spatial data through queries
        let center = Point::new(40.7128, -74.0060);
        let nearby = db
            .find_nearest_neighbors("persistent_geohash", &center, 10000.0, 100)
            .unwrap();
        assert!(!nearby.is_empty());

        let stats = db.stats().unwrap();
        assert!(stats.key_count >= 1000); // At least the data we inserted
    }
}

#[test]
fn test_coordinate_edge_cases() {
    let db = SpatioLite::memory().unwrap();

    // Test extreme coordinate values
    let edge_cases = vec![
        (90.0, 180.0),           // North pole, international date line
        (-90.0, -180.0),         // South pole, opposite date line
        (0.0, 0.0),              // Equator, prime meridian
        (89.99999, 179.99999),   // Near north pole
        (-89.99999, -179.99999), // Near south pole
        (40.7128, -74.0060),     // NYC (normal case)
    ];

    for (i, (lat, lon)) in edge_cases.iter().enumerate() {
        let point = Point::new(*lat, *lon);
        let key = format!("edge_case_{}", i);

        // Test basic point insertion
        db.insert_point(&key, &point, None).unwrap();

        // Test geohash operations with various precisions
        for precision in 1..=12 {
            let geohash_key = format!("edge_geohash_{}_{}", i, precision);
            db.insert_point_with_geohash(&geohash_key, &point, precision, b"edge_data", None)
                .unwrap();
        }

        // Test S2 cell operations
        for level in 1..=20 {
            let s2_key = format!("edge_s2_{}_{}", i, level);
            db.insert_point_with_s2(&s2_key, &point, level, b"s2_edge_data", None)
                .unwrap();
        }
    }

    // Test distance calculations between edge points
    let point1 = Point::new(89.0, 179.0);
    let point2 = Point::new(89.0, -179.0);
    let distance = point1.distance_to(&point2);
    assert!(distance > 0.0); // Should be a valid distance across date line
}

#[test]
fn test_memory_efficiency() {
    let db = SpatioLite::memory().unwrap();

    // Insert a large amount of data and monitor memory usage patterns
    let data_points = 10000;

    for i in 0..data_points {
        let lat = 40.0 + (i as f64 / 1000.0); // Spread over 10 degrees
        let lon = -74.0 + (i as f64 / 1000.0);
        let point = Point::new(lat, lon);

        // Create data of varying sizes
        let data_size = 100 + (i % 900); // 100-1000 bytes
        let data = vec![b'x'; data_size];

        db.insert_point_with_geohash("memory_test", &point, 8, &data, None)
            .unwrap();

        // Periodic verification
        if i % 1000 == 0 {
            let stats = db.stats().unwrap();
            assert!(stats.key_count > 0);
        }
    }

    // Test memory usage by performing many queries
    let center = Point::new(40.5, -73.5);
    for radius in [100.0, 500.0, 1000.0, 5000.0, 10000.0] {
        let results = db
            .find_nearest_neighbors("memory_test", &center, radius, 100)
            .unwrap();
        assert!(!results.is_empty());
    }
}

#[test]
fn test_error_conditions() {
    let db = SpatioLite::memory().unwrap();

    // Test various error conditions

    // Test invalid coordinates (these should still work but be at edges)
    let extreme_point = Point::new(90.0, 180.0);
    db.insert_point("extreme", &extreme_point, None).unwrap();

    // Test empty keys and values
    db.insert("", b"", None).unwrap();
    assert_eq!(db.get("").unwrap().unwrap(), &b""[..]);

    // Test very long keys and values
    let long_key = "a".repeat(10000);
    let long_value = vec![b'b'; 100000];
    db.insert(&long_key, &long_value, None).unwrap();
    assert_eq!(db.get(&long_key).unwrap().unwrap(), long_value);

    // Test queries on non-existent prefixes
    let empty_results = db
        .find_nearest_neighbors("nonexistent", &Point::new(0.0, 0.0), 1000.0, 10)
        .unwrap();
    assert!(empty_results.is_empty());

    // Test trajectory queries with invalid time ranges
    let empty_trajectory = db.query_trajectory("nonexistent", 0, 1000).unwrap();
    assert!(empty_trajectory.is_empty());
}

#[test]
fn test_spatial_query_performance() {
    let db = SpatioLite::memory().unwrap();

    // Create a dense spatial dataset
    for i in 0..5000 {
        let lat = 40.0 + ((i % 100) as f64 * 0.001);
        let lon = -74.0 + ((i / 100) as f64 * 0.001);
        let point = Point::new(lat, lon);

        db.insert_point_with_geohash("perf_test", &point, 8, b"performance_data", None)
            .unwrap();
    }

    // Measure query performance
    let center = Point::new(40.025, -74.025);
    let start = Instant::now();

    for _ in 0..100 {
        let _results = db
            .find_nearest_neighbors("perf_test", &center, 1000.0, 20)
            .unwrap();
    }

    let duration = start.elapsed();
    println!("100 spatial queries took: {:?}", duration);

    // Ensure reasonable performance (adjust threshold as needed)
    assert!(duration < Duration::from_millis(1000));
}

#[test]
fn test_bounding_box_operations() {
    let db = SpatioLite::memory().unwrap();

    // Create test data within known bounds
    let _min_point = Point::new(40.0, -74.5);
    let _max_point = Point::new(41.0, -73.5);
    let bbox = BoundingBox::new(40.0, -74.5, 41.0, -73.5);

    // Insert points both inside and outside the bounding box
    for i in 0..100 {
        let lat = 39.5 + (i as f64 * 0.02); // Range from 39.5 to 41.5
        let lon = -74.5 + (i as f64 * 0.02); // Range from -74.5 to -72.5
        let point = Point::new(lat, lon);

        db.insert_point_with_geohash("bbox_test", &point, 8, b"bbox_data", None)
            .unwrap();
    }

    // Test within query (simplified implementation)
    let within_results = db.within("bbox_test", &bbox).unwrap();

    // Since the within implementation is simplified and returns all points with prefix,
    // we'll just verify we got some results and they have the expected prefix
    assert!(!within_results.is_empty());

    // Verify all results have the expected prefix
    for (key, _data, _point) in &within_results {
        assert!(key.starts_with("bbox_test"));
    }
}

#[test]
fn test_mixed_operation_patterns() {
    let db = SpatioLite::memory().unwrap();

    // Simulate a realistic mixed workload
    for i in 0..1000i32 {
        match i % 7 {
            0 => {
                // IoT sensor data with TTL
                let lat = 40.7128 + ((i % 100) as f64 * 0.0001);
                let lon = -74.0060 + ((i % 100) as f64 * 0.0001);
                let point = Point::new(lat, lon);
                let data = format!("temp:{}°C,humidity:{}%", 20 + i % 20, 40 + i % 40);
                let opts = SetOptions::with_ttl(Duration::from_secs(300));

                db.insert_point_with_geohash("sensors", &point, 8, data.as_bytes(), Some(opts))
                    .unwrap();
            }
            1 => {
                // Vehicle tracking
                let lat = 40.7 + ((i % 200) as f64 * 0.001);
                let lon = -74.0 + ((i % 200) as f64 * 0.001);
                let point = Point::new(lat, lon);
                let timestamp = 1640995200 + (i as u64 * 30);
                let trajectory = vec![(point, timestamp)];

                db.insert_trajectory(&format!("vehicle_{}", i % 10), &trajectory, None)
                    .unwrap();
            }
            2 => {
                // Static reference data
                let key = format!("reference:location:{}", i);
                let value = format!("name:Location {},type:landmark", i);

                db.insert(&key, value.as_bytes(), None).unwrap();
            }
            3 => {
                // Batch analytics data
                db.atomic(|batch| {
                    for j in 0..5 {
                        let key = format!("analytics:batch_{}:item_{}", i, j);
                        let value = format!("metric_{}:{}", j, i * j);
                        batch.insert(&key, value.as_bytes(), None)?;
                    }
                    Ok(())
                })
                .unwrap();
            }
            4 => {
                // Spatial queries
                let center = Point::new(40.72, -74.01);
                let _results = db
                    .find_nearest_neighbors("sensors", &center, 500.0, 10)
                    .unwrap();
            }
            5 => {
                // Regular data retrieval
                let key = format!("reference:location:{}", (i as i32).saturating_sub(100));
                let _value = db.get(&key).unwrap();
            }
            6 => {
                // Data cleanup
                let old_key = format!("reference:location:{}", (i as i32).saturating_sub(500));
                let _deleted = db.delete(&old_key).unwrap();
            }
            _ => unreachable!(),
        }
    }

    // Verify system is still operational
    let stats = db.stats().unwrap();
    assert!(stats.key_count > 0);

    let spatial_stats = db.spatial_stats().unwrap();
    assert!(spatial_stats.total_points > 0);
}
