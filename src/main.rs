use spatio_lite::{SetOptions, SpatioLite};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SpatioLite Demo");

    // Create an in-memory database
    let db = SpatioLite::memory()?;

    // Basic key-value operations
    println!("\n📝 Basic Operations:");

    // Insert some location data
    db.insert(
        "uav:drone_1",
        &b"lat:40.7128,lon:-74.0060,alt:100"[..],
        None,
    )?;
    db.insert(
        "uav:drone_2",
        &b"lat:40.7589,lon:-73.9851,alt:150"[..],
        None,
    )?;

    // Get values
    if let Some(location) = db.get("uav:drone_1")? {
        println!("Drone 1 location: {}", String::from_utf8_lossy(&location));
    }

    // Atomic batch operations
    println!("\n🔄 Atomic Batch Operations:");
    db.atomic(|batch| {
        batch.insert("sensor:temp_1", &b"temperature:22.5,unit:celsius"[..], None)?;
        batch.insert("sensor:temp_2", &b"temperature:23.1,unit:celsius"[..], None)?;
        batch.insert(
            "sensor:humidity_1",
            &b"humidity:65.2,unit:percent"[..],
            None,
        )?;
        Ok(())
    })?;

    // TTL (Time-To-Live) example
    println!("\n⏰ TTL Example:");
    let opts = SetOptions::with_ttl(Duration::from_secs(2));
    db.insert("temp:session", &b"temporary_data"[..], Some(opts))?;

    println!("Data exists: {}", db.get("temp:session")?.is_some());

    // Wait for expiration
    std::thread::sleep(Duration::from_secs(3));
    println!(
        "After TTL expiration: {}",
        db.get("temp:session")?.is_some()
    );

    // Delete operations
    println!("\n🗑️  Delete Operations:");
    if let Some(deleted) = db.delete("uav:drone_2")? {
        println!("Deleted: {}", String::from_utf8_lossy(&deleted));
    }

    // Check stats
    println!("\n📊 Database Stats:");
    let stats = db.stats()?;
    println!("Keys in database: {}", stats.key_count);
    println!("Expired items cleaned: {}", stats.expired_count);

    // Demonstrate persistence (commented out for demo)
    /*
    println!("\n💾 Persistence Example:");
    {
        let persistent_db = SpatioLite::open("demo.aof")?;
        persistent_db.insert("persistent:key", &b"This will survive restarts"[..], None)?;
        persistent_db.sync()?; // Force sync to disk
    } // Database closes here

    // Reopen and verify persistence
    {
        let reopened_db = SpatioLite::open("demo.aof")?;
        if let Some(value) = reopened_db.get("persistent:key")? {
            println!("Persisted value: {}", String::from_utf8_lossy(&value));
        }
    }
    */

    println!("\n✅ Demo completed successfully!");
    Ok(())
}
