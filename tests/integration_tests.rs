use spatio_lite::{Config, Rect, SetOptions, SpatioLite, SyncPolicy};
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn test_basic_usage() {
    let db = SpatioLite::memory().unwrap();

    // Single atomic insert
    db.insert("location:123", &b"lat:40.7128,lon:-74.0060"[..], None)
        .unwrap();

    // Get the value
    let value = db.get("location:123").unwrap().unwrap();
    assert_eq!(value, &b"lat:40.7128,lon:-74.0060"[..]);

    // Delete
    let deleted = db.delete("location:123").unwrap().unwrap();
    assert_eq!(deleted, &b"lat:40.7128,lon:-74.0060"[..]);

    // Should be gone now
    assert!(db.get("location:123").unwrap().is_none());
}

#[test]
fn test_atomic_batch() {
    let db = SpatioLite::memory().unwrap();

    // Atomic batch of operations
    db.atomic(|batch| {
        batch.insert("uav:1", &b"40.7128,-74.0060,100"[..], None)?;
        batch.insert("uav:2", &b"40.7589,-73.9851,150"[..], None)?;
        batch.insert("uav:3", &b"40.6892,-74.0445,200"[..], None)?;
        Ok(())
    })
    .unwrap();

    // All items should be present
    assert!(db.get("uav:1").unwrap().is_some());
    assert!(db.get("uav:2").unwrap().is_some());
    assert!(db.get("uav:3").unwrap().is_some());
}

#[test]
fn test_ttl() {
    let db = SpatioLite::memory().unwrap();

    // Insert with TTL
    let opts = SetOptions::with_ttl(Duration::from_millis(100));
    db.insert("temp:data", &b"expires_soon"[..], Some(opts))
        .unwrap();

    // Should exist initially
    assert!(db.get("temp:data").unwrap().is_some());

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(150));

    // Should be expired now
    assert!(db.get("temp:data").unwrap().is_none());
}

#[test]
fn test_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create database with persistence
    {
        let db = SpatioLite::open(path).unwrap();
        db.insert("persistent:key", &b"persistent_value"[..], None)
            .unwrap();
        db.sync().unwrap();
    } // Database closes here

    // Reopen and verify data persisted
    {
        let db = SpatioLite::open(path).unwrap();
        let value = db.get("persistent:key").unwrap().unwrap();
        assert_eq!(value, &b"persistent_value"[..]);
    }
}

#[test]
fn test_spatial_rect() {
    // Test rectangle creation and operations
    let rect1 = Rect::new(vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();
    let rect2 = Rect::new(vec![5.0, 5.0], vec![15.0, 15.0]).unwrap();

    assert!(rect1.intersects(&rect2));
    assert!(rect1.contains_point(&[5.0, 5.0]));
    assert!(!rect1.contains_point(&[15.0, 15.0]));

    let point = Rect::point(vec![7.5, 7.5]);
    assert!(rect1.intersects(&point));
    assert!(rect2.intersects(&point));
}

#[test]
fn test_config() {
    let mut config = Config::default();
    config.sync_policy = SyncPolicy::Always;
    config.auto_shrink_disabled = true;
    config.max_dimensions = 3;

    let db = SpatioLite::memory().unwrap();
    db.set_config(config.clone()).unwrap();

    let retrieved_config = db.config().unwrap();
    assert_eq!(retrieved_config.sync_policy, SyncPolicy::Always);
    assert!(retrieved_config.auto_shrink_disabled);
    assert_eq!(retrieved_config.max_dimensions, 3);
}

#[test]
fn test_stats() {
    let db = SpatioLite::memory().unwrap();

    let initial_stats = db.stats().unwrap();
    assert_eq!(initial_stats.key_count, 0);

    // Add some data
    db.insert("key1", &b"value1"[..], None).unwrap();
    db.insert("key2", &b"value2"[..], None).unwrap();

    let stats = db.stats().unwrap();
    assert_eq!(stats.key_count, 2);
}

#[test]
fn test_spatio_temporal_workflow() {
    let db = SpatioLite::memory().unwrap();

    // Simulate UAV tracking with timestamps
    let timestamp_1 = std::time::SystemTime::now();
    let timestamp_2 = timestamp_1 + Duration::from_secs(30);
    let _timestamp_3 = timestamp_2 + Duration::from_secs(30);

    // Insert location data with timestamps as part of the key
    db.atomic(|batch| {
        batch.insert(
            "uav:drone_42:1640995200", // Unix timestamp in key
            &b"40.7128,-74.0060,100,2022-01-01T10:00:00Z"[..],
            None,
        )?;
        batch.insert(
            "uav:drone_42:1640995230",
            &b"40.7150,-74.0040,105,2022-01-01T10:00:30Z"[..],
            None,
        )?;
        batch.insert(
            "uav:drone_42:1640995260",
            &b"40.7172,-74.0020,110,2022-01-01T10:01:00Z"[..],
            None,
        )?;
        Ok(())
    })
    .unwrap();

    // Verify all trajectory points are stored
    assert!(db.get("uav:drone_42:1640995200").unwrap().is_some());
    assert!(db.get("uav:drone_42:1640995230").unwrap().is_some());
    assert!(db.get("uav:drone_42:1640995260").unwrap().is_some());

    // Insert sensor data with TTL (temporary readings)
    let sensor_opts = SetOptions::with_ttl(Duration::from_millis(200));
    db.insert(
        "sensor:temp:building_a",
        &b"22.5,celsius,2022-01-01T10:00:00Z"[..],
        Some(sensor_opts),
    )
    .unwrap();

    // Verify sensor data exists
    assert!(db.get("sensor:temp:building_a").unwrap().is_some());

    // Wait for sensor data to expire
    std::thread::sleep(Duration::from_millis(250));
    assert!(db.get("sensor:temp:building_a").unwrap().is_none());

    // UAV trajectory data should still be there
    assert!(db.get("uav:drone_42:1640995200").unwrap().is_some());
}

#[test]
fn test_large_batch_operations() {
    let db = SpatioLite::memory().unwrap();

    // Test larger batch operations
    db.atomic(|batch| {
        for i in 0..1000 {
            let key = format!("sensor:{}:reading", i);
            let value = format!("temp:20.{},humidity:{}%", i % 100, (i * 13) % 100);
            batch.insert(key.as_str(), value.as_bytes(), None)?;
        }
        Ok(())
    })
    .unwrap();

    // Verify some random entries
    assert!(db.get("sensor:0:reading").unwrap().is_some());
    assert!(db.get("sensor:500:reading").unwrap().is_some());
    assert!(db.get("sensor:999:reading").unwrap().is_some());

    let stats = db.stats().unwrap();
    assert_eq!(stats.key_count, 1000); // 1000 from this test
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let db = Arc::new(SpatioLite::memory().unwrap());
    let mut handles = vec![];

    // Spawn multiple threads to test concurrent access
    for thread_id in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key = format!("thread:{}:item:{}", thread_id, i);
                let value = format!("data_from_thread_{}_item_{}", thread_id, i);

                db_clone
                    .insert(key.as_str(), value.as_bytes(), None)
                    .unwrap();

                // Verify we can read what we just wrote
                let retrieved = db_clone.get(&key).unwrap().unwrap();
                assert_eq!(retrieved, value.as_bytes());
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
    assert_eq!(stats.key_count, 500); // 500 from threads in this test
}

#[test]
fn test_edge_cases() {
    let db = SpatioLite::memory().unwrap();

    // Test empty key/value
    db.insert("", &b""[..], None).unwrap();
    assert_eq!(db.get("").unwrap().unwrap(), &b""[..]);

    // Test large values
    let large_value = vec![42u8; 1024 * 1024]; // 1MB
    db.insert("large_key", large_value.as_slice(), None)
        .unwrap();
    assert_eq!(
        db.get("large_key").unwrap().unwrap(),
        large_value.as_slice()
    );

    // Test unicode keys
    db.insert("🚁:位置", &b"unicode_test"[..], None).unwrap();
    assert_eq!(db.get("🚁:位置").unwrap().unwrap(), &b"unicode_test"[..]);

    // Test deletion of non-existent key
    assert!(db.delete("non_existent").unwrap().is_none());
}
