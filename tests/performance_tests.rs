//! Performance tests for SpatioLite
//!
//! These tests are designed to be CI-friendly with generous timing thresholds
//! to account for variable performance in CI environments while still catching
//! significant performance regressions.

use spatio_lite::{Point, SetOptions, SpatioLite};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

#[test]
fn test_insertion_performance() {
    let db = SpatioLite::memory().unwrap();
    let num_inserts = 10000;

    let start = Instant::now();
    for i in 0..num_inserts {
        let key = format!("perf_key_{}", i);
        let value = format!("perf_value_{}", i);
        db.insert(&key, value.as_bytes(), None).unwrap();
    }
    let duration = start.elapsed();

    println!("Inserted {} items in {:?}", num_inserts, duration);
    println!(
        "Rate: {:.2} ops/sec",
        num_inserts as f64 / duration.as_secs_f64()
    );

    // Verify all items were inserted
    let stats = db.stats().unwrap();
    assert_eq!(stats.key_count, num_inserts);

    // Performance should be reasonable (CI-friendly threshold)
    assert!(
        duration < Duration::from_secs(30),
        "Insertion took too long: {:?}",
        duration
    );
}

#[test]
fn test_spatial_insertion_performance() {
    let db = SpatioLite::memory().unwrap();
    let num_points = 5000;

    let start = Instant::now();
    for i in 0..num_points {
        let lat = 40.7128 + (i as f64 * 0.0001);
        let lon = -74.0060 + (i as f64 * 0.0001);
        let point = Point::new(lat, lon);
        let data = format!("spatial_data_{}", i);

        db.insert_point_with_geohash("spatial_perf", &point, 8, data.as_bytes(), None)
            .unwrap();
    }
    let duration = start.elapsed();

    println!("Inserted {} spatial points in {:?}", num_points, duration);
    println!(
        "Spatial rate: {:.2} ops/sec",
        num_points as f64 / duration.as_secs_f64()
    );

    // Test spatial query performance
    let query_start = Instant::now();
    let center = Point::new(40.7128, -74.0060);
    let results = db
        .find_nearest_neighbors("spatial_perf", &center, 1000.0, 100)
        .unwrap();
    let query_duration = query_start.elapsed();

    println!(
        "Spatial query took {:?}, found {} results",
        query_duration,
        results.len()
    );
    assert!(!results.is_empty());
}

#[test]
fn test_batch_operation_performance() {
    let db = SpatioLite::memory().unwrap();
    let num_batches = 100;
    let items_per_batch = 100;

    let start = Instant::now();
    for batch_id in 0..num_batches {
        db.atomic(|batch| {
            for item_id in 0..items_per_batch {
                let key = format!("batch_{}:item_{}", batch_id, item_id);
                let value = format!("batch_value_{}_{}", batch_id, item_id);
                batch.insert(&key, value.as_bytes(), None)?;
            }
            Ok(())
        })
        .unwrap();
    }
    let duration = start.elapsed();

    let total_items = num_batches * items_per_batch;
    println!(
        "Inserted {} items in {} batches in {:?}",
        total_items, num_batches, duration
    );
    println!(
        "Batch rate: {:.2} ops/sec",
        total_items as f64 / duration.as_secs_f64()
    );

    let stats = db.stats().unwrap();
    assert_eq!(stats.key_count, total_items);
}

#[test]
fn test_read_performance() {
    let db = SpatioLite::memory().unwrap();
    let num_items = 10000;

    // Pre-populate database
    for i in 0..num_items {
        let key = format!("read_test_{}", i);
        let value = format!("read_value_{}", i);
        db.insert(&key, value.as_bytes(), None).unwrap();
    }

    // Test sequential reads
    let start = Instant::now();
    for i in 0..num_items {
        let key = format!("read_test_{}", i);
        let _value = db.get(&key).unwrap().unwrap();
    }
    let duration = start.elapsed();

    println!("Read {} items in {:?}", num_items, duration);
    println!(
        "Read rate: {:.2} ops/sec",
        num_items as f64 / duration.as_secs_f64()
    );

    // Test random reads
    let random_start = Instant::now();
    for i in 0..1000 {
        let random_id = (i * 7) % num_items; // Simple pseudo-random
        let key = format!("read_test_{}", random_id);
        let _value = db.get(&key).unwrap().unwrap();
    }
    let random_duration = random_start.elapsed();

    println!("Random read 1000 items in {:?}", random_duration);
    println!(
        "Random read rate: {:.2} ops/sec",
        1000.0 / random_duration.as_secs_f64()
    );
}

#[test]
fn test_concurrent_write_performance() {
    let db = Arc::new(SpatioLite::memory().unwrap());
    let num_threads = 8;
    let writes_per_thread = 1000;

    let start = Instant::now();
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let db_clone = Arc::clone(&db);
            thread::spawn(move || {
                for i in 0..writes_per_thread {
                    let key = format!("concurrent_{}_{}", thread_id, i);
                    let value = format!("concurrent_value_{}_{}", thread_id, i);
                    db_clone.insert(&key, value.as_bytes(), None).unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();

    let total_writes = num_threads * writes_per_thread;
    println!(
        "Concurrent: {} threads wrote {} items in {:?}",
        num_threads, total_writes, duration
    );
    println!(
        "Concurrent write rate: {:.2} ops/sec",
        total_writes as f64 / duration.as_secs_f64()
    );

    let stats = db.stats().unwrap();
    assert_eq!(stats.key_count, total_writes);
}

#[test]
fn test_concurrent_read_performance() {
    let db = Arc::new(SpatioLite::memory().unwrap());
    let num_items = 5000;
    let num_readers = 8;
    let reads_per_reader = 1000;

    // Pre-populate
    for i in 0..num_items {
        let key = format!("concurrent_read_{}", i);
        let value = format!("concurrent_read_value_{}", i);
        db.insert(&key, value.as_bytes(), None).unwrap();
    }

    let start = Instant::now();
    let handles: Vec<_> = (0..num_readers)
        .map(|reader_id| {
            let db_clone = Arc::clone(&db);
            thread::spawn(move || {
                for i in 0..reads_per_reader {
                    let item_id = (reader_id * 1000 + i) % num_items;
                    let key = format!("concurrent_read_{}", item_id);
                    let _value = db_clone.get(&key).unwrap().unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();

    let total_reads = num_readers * reads_per_reader;
    println!(
        "Concurrent: {} readers read {} items in {:?}",
        num_readers, total_reads, duration
    );
    println!(
        "Concurrent read rate: {:.2} ops/sec",
        total_reads as f64 / duration.as_secs_f64()
    );
}

#[test]
fn test_mixed_workload_performance() {
    let db = Arc::new(SpatioLite::memory().unwrap());
    let num_threads = 6;
    let operations_per_thread = 1000;

    let start = Instant::now();
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let db_clone = Arc::clone(&db);
            thread::spawn(move || {
                for i in 0..operations_per_thread {
                    match (thread_id + i) % 5 {
                        0 | 1 => {
                            // 40% writes
                            let key = format!("mixed_{}_{}", thread_id, i);
                            let value = format!("mixed_value_{}_{}", thread_id, i);
                            db_clone.insert(&key, value.as_bytes(), None).unwrap();
                        }
                        2 | 3 => {
                            // 40% reads
                            let read_id = (i + thread_id * 100) % (operations_per_thread / 2);
                            let key = format!("mixed_{}_{}", thread_id, read_id);
                            let _ = db_clone.get(&key).unwrap();
                        }
                        4 => {
                            // 20% spatial operations
                            let lat = 40.0 + (thread_id as f64 * 0.01) + (i as f64 * 0.0001);
                            let lon = -74.0 + (thread_id as f64 * 0.01) + (i as f64 * 0.0001);
                            let point = Point::new(lat, lon);
                            let data = format!("spatial_{}_{}", thread_id, i);

                            db_clone
                                .insert_point_with_geohash(
                                    &format!("spatial_{}", thread_id),
                                    &point,
                                    8,
                                    data.as_bytes(),
                                    None,
                                )
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();

    let total_ops = num_threads * operations_per_thread;
    println!(
        "Mixed workload: {} ops across {} threads in {:?}",
        total_ops, num_threads, duration
    );
    println!(
        "Mixed workload rate: {:.2} ops/sec",
        total_ops as f64 / duration.as_secs_f64()
    );
}

#[test]
fn test_persistence_performance() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let num_items = 5000;

    // Test write performance with persistence
    let write_start = Instant::now();
    {
        let db = SpatioLite::open(path).unwrap();

        for i in 0..num_items {
            let key = format!("persist_key_{}", i);
            let value = format!("persist_value_{}", i);
            db.insert(&key, value.as_bytes(), None).unwrap();

            // Sync every 100 items to test sync performance
            if i % 100 == 0 {
                db.sync().unwrap();
            }
        }

        // Final sync
        db.sync().unwrap();
    }
    let write_duration = write_start.elapsed();

    println!(
        "Persistent write: {} items in {:?}",
        num_items, write_duration
    );
    println!(
        "Persistent write rate: {:.2} ops/sec",
        num_items as f64 / write_duration.as_secs_f64()
    );

    // Test read performance after restart
    let read_start = Instant::now();
    {
        let db = SpatioLite::open(path).unwrap();

        for i in 0..num_items {
            let key = format!("persist_key_{}", i);
            let _value = db.get(&key).unwrap().unwrap();
        }
    }
    let read_duration = read_start.elapsed();

    println!(
        "Persistent read after restart: {} items in {:?}",
        num_items, read_duration
    );
    println!(
        "Persistent read rate: {:.2} ops/sec",
        num_items as f64 / read_duration.as_secs_f64()
    );
}

#[test]
fn test_large_dataset_performance() {
    let _db = SpatioLite::memory().unwrap();
    let dataset_sizes = vec![1000, 5000, 10000, 20000];

    for &size in &dataset_sizes {
        // Clear previous data by creating a new database
        let db = SpatioLite::memory().unwrap();

        let insert_start = Instant::now();
        for i in 0..size {
            let lat = 40.0 + (i as f64 * 0.0001);
            let lon = -74.0 + (i as f64 * 0.0001);
            let point = Point::new(lat, lon);
            let data = format!("large_dataset_item_{}", i);

            db.insert_point_with_geohash("large_dataset", &point, 8, data.as_bytes(), None)
                .unwrap();
        }
        let insert_duration = insert_start.elapsed();

        // Test query performance on large dataset
        let query_start = Instant::now();
        let center = Point::new(40.05, -73.95);
        let results = db
            .find_nearest_neighbors("large_dataset", &center, 1000.0, 100)
            .unwrap();
        let query_duration = query_start.elapsed();

        println!(
            "Dataset size {}: Insert {:?} ({:.2} ops/sec), Query {:?} ({} results)",
            size,
            insert_duration,
            size as f64 / insert_duration.as_secs_f64(),
            query_duration,
            results.len()
        );

        // Verify reasonable performance scaling (CI-friendly thresholds)
        assert!(
            insert_duration < Duration::from_secs(60),
            "Insert for size {} took too long: {:?}",
            size,
            insert_duration
        );
        assert!(
            query_duration < Duration::from_secs(1),
            "Query for size {} took too long: {:?}",
            size,
            query_duration
        );
        assert!(!results.is_empty());
    }
}

#[test]
fn test_ttl_performance_impact() {
    let db = SpatioLite::memory().unwrap();
    let num_items = 5000;

    // Test without TTL
    let no_ttl_start = Instant::now();
    for i in 0..num_items {
        let key = format!("no_ttl_{}", i);
        let value = format!("no_ttl_value_{}", i);
        db.insert(&key, value.as_bytes(), None).unwrap();
    }
    let no_ttl_duration = no_ttl_start.elapsed();

    // Test with TTL
    let ttl_start = Instant::now();
    for i in 0..num_items {
        let key = format!("with_ttl_{}", i);
        let value = format!("with_ttl_value_{}", i);
        let opts = SetOptions::with_ttl(Duration::from_secs(60));
        db.insert(&key, value.as_bytes(), Some(opts)).unwrap();
    }
    let ttl_duration = ttl_start.elapsed();

    println!(
        "No TTL: {} items in {:?} ({:.2} ops/sec)",
        num_items,
        no_ttl_duration,
        num_items as f64 / no_ttl_duration.as_secs_f64()
    );

    println!(
        "With TTL: {} items in {:?} ({:.2} ops/sec)",
        num_items,
        ttl_duration,
        num_items as f64 / ttl_duration.as_secs_f64()
    );

    // TTL should not significantly impact performance
    let overhead_ratio = ttl_duration.as_secs_f64() / no_ttl_duration.as_secs_f64();
    println!("TTL overhead ratio: {:.2}x", overhead_ratio);

    // Be more lenient in CI environments where performance can be highly variable
    // The main goal is to ensure TTL doesn't cause catastrophic slowdown
    assert!(
        overhead_ratio < 10.0,
        "TTL overhead ratio {:.2}x is too high",
        overhead_ratio
    );

    // Also verify both operations completed in reasonable time
    assert!(
        no_ttl_duration < Duration::from_secs(30),
        "No-TTL operations took too long: {:?}",
        no_ttl_duration
    );
    assert!(
        ttl_duration < Duration::from_secs(60),
        "TTL operations took too long: {:?}",
        ttl_duration
    );
}

#[test]
fn test_memory_usage_under_load() {
    let db = SpatioLite::memory().unwrap();

    // Continuously add and remove data to test memory management
    for cycle in 0..10 {
        let cycle_start = Instant::now();

        // Add data
        for i in 0..1000 {
            let key = format!("memory_test_{}_{}", cycle, i);
            let value = vec![b'x'; 1000]; // 1KB per item
            db.insert(&key, &value, None).unwrap();
        }

        // Query data
        let stats = db.stats().unwrap();

        // Remove old data from previous cycles
        if cycle > 0 {
            for i in 0..1000 {
                let old_key = format!("memory_test_{}_{}", cycle - 1, i);
                let _ = db.delete(&old_key).unwrap();
            }
        }

        let cycle_duration = cycle_start.elapsed();
        println!(
            "Memory cycle {}: {:?}, {} keys",
            cycle, cycle_duration, stats.key_count
        );
    }

    // Final verification
    let final_stats = db.stats().unwrap();
    assert!(final_stats.key_count <= 2000); // Should not accumulate too much data
}

#[test]
fn test_spatial_query_scaling() {
    let db = SpatioLite::memory().unwrap();
    let base_points = 1000;

    // Insert spatial data
    for i in 0..base_points {
        let lat = 40.0 + (i as f64 * 0.001);
        let lon = -74.0 + (i as f64 * 0.001);
        let point = Point::new(lat, lon);
        let data = format!("scaling_test_{}", i);

        db.insert_point_with_geohash("scaling_test", &point, 8, data.as_bytes(), None)
            .unwrap();
    }

    // Test query performance with different result set sizes
    let center = Point::new(40.5, -74.5);
    let radii = vec![100.0, 500.0, 1000.0, 5000.0, 10000.0];
    let limits = vec![10, 50, 100, 500, 1000];

    for &radius in &radii {
        for &limit in &limits {
            let query_start = Instant::now();
            let results = db
                .find_nearest_neighbors("scaling_test", &center, radius, limit)
                .unwrap();
            let query_duration = query_start.elapsed();

            println!(
                "Query radius={}, limit={}: {:?} -> {} results",
                radius,
                limit,
                query_duration,
                results.len()
            );

            // Verify reasonable performance (CI-friendly threshold)
            assert!(
                query_duration < Duration::from_secs(5),
                "Query radius={}, limit={} took too long: {:?}",
                radius,
                limit,
                query_duration
            );
        }
    }
}
