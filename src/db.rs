use crate::batch::AtomicBatch;
use crate::error::{Result, SpatioLiteError};
use crate::geometry::{Coordinate, Geometry, GeometryOps, LineString, Polygon};
use crate::index::IndexManager;
use crate::persistence::AOFFile;
use crate::spatial::{BoundingBox, Point, SpatialKey};
use crate::types::{Config, DbItem, DbStats, SetOptions};
use bytes::Bytes;
use geohash;
use std::collections::{BTreeMap, HashMap};

use std::path::Path;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use std::time::{Duration, SystemTime};

/// Main database structure
#[derive(Clone)]
pub struct DB {
    /// Read-write lock for the entire database
    inner: Arc<RwLock<DBInner>>,
}

pub(crate) struct DBInner {
    /// Main key-value storage (B-tree for ordered access)
    pub keys: BTreeMap<Bytes, DbItem>,

    /// Items ordered by expiration time
    pub expirations: BTreeMap<SystemTime, Vec<Bytes>>,

    /// Index manager
    pub index_manager: IndexManager,

    /// Append-only file for persistence
    pub aof_file: Option<AOFFile>,

    /// Database configuration
    pub config: Config,

    /// Whether the database persists to disk
    #[allow(dead_code)]
    pub persist: bool,

    /// Whether the database is closed
    pub closed: bool,

    /// Database statistics
    pub stats: DbStats,

    /// Buffer for write operations
    #[allow(dead_code)]
    pub write_buffer: Vec<u8>,

    /// Whether a shrink operation is in progress
    pub shrinking: bool,

    /// Size of AOF file at last shrink
    pub last_aof_size: u64,
}

impl DB {
    /// Open a database at the given path
    /// Use ":memory:" for in-memory only database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let is_memory = path.to_str() == Some(":memory:");

        let mut inner = DBInner {
            keys: BTreeMap::new(),
            expirations: BTreeMap::new(),
            index_manager: IndexManager::new(),
            aof_file: None,
            config: Config::default(),
            persist: !is_memory,
            closed: false,
            stats: DbStats::default(),
            write_buffer: Vec::new(),
            shrinking: false,
            last_aof_size: 0,
        };

        // Initialize persistence if not in-memory
        if !is_memory {
            let aof_file = AOFFile::open(path)?;
            inner.load_from_aof(&aof_file)?;
            inner.aof_file = Some(aof_file);
        }

        let db = DB {
            inner: Arc::new(RwLock::new(inner)),
        };

        // Start background tasks
        db.start_background_tasks();

        Ok(db)
    }

    /// Create an in-memory database
    pub fn memory() -> Result<Self> {
        Self::open(":memory:")
    }

    /// Get the current configuration
    pub fn config(&self) -> Result<Config> {
        let inner = self.read()?;
        Ok(inner.config.clone())
    }

    /// Update the database configuration
    pub fn set_config(&self, config: Config) -> Result<()> {
        let mut inner = self.write()?;
        inner.config = config;
        Ok(())
    }

    /// Get database statistics
    pub fn stats(&self) -> Result<DbStats> {
        let inner = self.read()?;
        Ok(inner.stats.clone())
    }

    /// Insert a single key-value pair atomically
    pub fn insert(
        &self,
        key: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
        opts: Option<SetOptions>,
    ) -> Result<Option<Bytes>> {
        let key = Bytes::copy_from_slice(key.as_ref());
        let value = Bytes::copy_from_slice(value.as_ref());

        let mut inner = self.write()?;
        if inner.closed {
            return Err(SpatioLiteError::DatabaseClosed);
        }

        let item = if let Some(ref opts) = opts {
            if let Some(ttl) = opts.ttl {
                DbItem::with_ttl(key.clone(), value.clone(), ttl)
            } else if let Some(expires_at) = opts.expires_at {
                DbItem::with_expiration(key.clone(), value.clone(), expires_at)
            } else {
                DbItem::new(key.clone(), value.clone())
            }
        } else {
            DbItem::new(key.clone(), value.clone())
        };

        let old_item = inner.insert_item(key.clone(), item);

        // Write to AOF if persisting
        if let Some(ref mut aof_file) = inner.aof_file {
            aof_file.write_set(&key, &value, opts.as_ref())?;
        }

        Ok(old_item.map(|item| item.value))
    }

    /// Get a value by key
    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Bytes>> {
        let key = Bytes::copy_from_slice(key.as_ref());
        let inner = self.read()?;

        if inner.closed {
            return Err(SpatioLiteError::DatabaseClosed);
        }

        match inner.get_item(&key) {
            Some(item) if !item.is_expired() => Ok(Some(item.value.clone())),
            _ => Ok(None),
        }
    }

    /// Delete a key atomically
    pub fn delete(&self, key: impl AsRef<[u8]>) -> Result<Option<Bytes>> {
        let key = Bytes::copy_from_slice(key.as_ref());
        let mut inner = self.write()?;

        if inner.closed {
            return Err(SpatioLiteError::DatabaseClosed);
        }

        let old_item = inner.remove_item(&key);

        // Write to AOF if persisting
        if let Some(ref mut aof_file) = inner.aof_file {
            aof_file.write_delete(&key)?;
        }

        Ok(old_item.map(|item| item.value))
    }

    /// Execute multiple operations atomically
    pub fn atomic<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut AtomicBatch) -> Result<R>,
    {
        let mut inner = self.write()?;
        if inner.closed {
            return Err(SpatioLiteError::DatabaseClosed);
        }

        let mut batch = AtomicBatch::new();
        let result = f(&mut batch)?;

        // Apply all operations in the batch atomically
        batch.apply(&mut inner)?;

        Ok(result)
    }

    /// Read-only access to the database
    #[allow(dead_code)]
    pub(crate) fn view<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&DBInner) -> Result<R>,
    {
        let inner = self.read()?;
        if inner.closed {
            return Err(SpatioLiteError::DatabaseClosed);
        }

        f(&inner)
    }

    /// Close the database
    pub fn close(&self) -> Result<()> {
        let mut inner = self.write()?;
        if inner.closed {
            return Ok(());
        }

        // Flush any pending writes
        if let Some(ref mut aof_file) = inner.aof_file {
            aof_file.flush()?;
        }

        inner.closed = true;
        Ok(())
    }

    /// Force a sync to disk
    pub fn sync(&self) -> Result<()> {
        let mut inner = self.write()?;
        if let Some(ref mut aof_file) = inner.aof_file {
            aof_file.sync()?;
        }
        Ok(())
    }

    /// Manually trigger a shrink operation
    pub fn shrink(&self) -> Result<()> {
        let mut inner = self.write()?;

        if inner.aof_file.is_none() {
            return Ok(());
        }

        inner.shrinking = true;

        // Create a new temporary AOF file for compaction
        let mut shrink_file = inner.aof_file.as_ref().unwrap().create_shrink_file()?;

        // Write all current live data to the shrink file
        for (key, item) in &inner.keys {
            // Skip expired items
            if let Some(expires_at) = item.expires_at {
                if expires_at <= std::time::SystemTime::now() {
                    continue;
                }
            }

            // Write the current key-value pair
            let opts = item.expires_at.map(|expires_at| crate::types::SetOptions {
                expires_at: Some(expires_at),
                ttl: None,
            });

            shrink_file.write_set(&key.clone(), &item.value, opts.as_ref())?;
        }

        // Flush the shrink file
        shrink_file.flush()?;

        // Replace the original AOF file with the compacted one
        let aof_file = inner.aof_file.as_mut().unwrap();
        aof_file.replace_with_shrink()?;

        // Update size tracking
        inner.last_aof_size = aof_file.size()?;
        inner.shrinking = false;

        Ok(())
    }

    /// Get a read lock on the inner data
    fn read(&self) -> Result<RwLockReadGuard<'_, DBInner>> {
        self.inner
            .read()
            .map_err(|_| SpatioLiteError::Lock("Failed to acquire read lock".to_string()))
    }

    /// Get a write lock on the inner data
    fn write(&self) -> Result<RwLockWriteGuard<'_, DBInner>> {
        self.inner
            .write()
            .map_err(|_| SpatioLiteError::Lock("Failed to acquire write lock".to_string()))
    }

    /// Start background tasks (expiration cleanup, auto-shrink, etc.)
    fn start_background_tasks(&self) {
        let db_weak = Arc::downgrade(&self.inner);

        thread::spawn(move || {
            let mut last_cleanup = SystemTime::now();
            let cleanup_interval = Duration::from_secs(1);

            loop {
                thread::sleep(cleanup_interval);

                if let Some(db_arc) = db_weak.upgrade() {
                    if let Ok(mut inner) = db_arc.write() {
                        if inner.closed {
                            break;
                        }

                        // Clean up expired items
                        if last_cleanup.elapsed().unwrap_or(Duration::ZERO) >= cleanup_interval {
                            inner.cleanup_expired();
                            last_cleanup = SystemTime::now();
                        }

                        // Auto-shrink if needed
                        if !inner.config.auto_shrink_disabled {
                            inner.maybe_auto_shrink();
                        }
                    }
                } else {
                    // Database has been dropped
                    break;
                }
            }
        });
    }
}

impl DBInner {
    /// Load data from AOF file
    fn load_from_aof(&mut self, aof_file: &AOFFile) -> Result<()> {
        use crate::persistence::AOFCommand;

        // Clone the AOF file for reading
        let mut aof_reader = AOFFile::open(aof_file.path())?;

        // Replay all commands from the AOF file
        aof_reader.replay(|command| {
            match command {
                AOFCommand::Set {
                    key,
                    value,
                    expires_at,
                } => {
                    // Create DbItem with expiration if specified
                    let item = if let Some(expires_at) = expires_at {
                        DbItem::with_expiration(key.clone(), value, expires_at)
                    } else {
                        DbItem::new(key.clone(), value)
                    };

                    // Insert the item
                    self.insert_item(key, item);
                }
                AOFCommand::Delete { key } => {
                    // Remove the key
                    self.remove_item(&key);
                }
                AOFCommand::Expire { key, expires_at } => {
                    // Set expiration for the key
                    if let Some(mut item) = self.keys.get(&key).cloned() {
                        item.expires_at = Some(expires_at);
                        self.insert_item(key.clone(), item);
                    }
                }
            }
            Ok(())
        })?;

        Ok(())
    }

    /// Clean up expired items
    fn cleanup_expired(&mut self) {
        let _now = SystemTime::now();
        let mut expired_keys = Vec::new();

        // Find all expired items
        for (key, item) in &self.keys {
            if item.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        // Remove expired items
        for key in expired_keys {
            if let Some(item) = self.keys.remove(&key) {
                // Remove from expiration index
                if let Some(expires_at) = item.expires_at {
                    if let Some(keys_at_time) = self.expirations.get_mut(&expires_at) {
                        keys_at_time.retain(|k| k != &key);
                        if keys_at_time.is_empty() {
                            self.expirations.remove(&expires_at);
                        }
                    }
                }

                // Remove from indexes
                self.index_manager.remove_item(&key, &item.value);
                self.stats.expired_count += 1;
            }
        }

        // Update key count
        self.stats.key_count = self.keys.len() as u64;
    }

    /// Check if auto-shrink should be triggered
    fn maybe_auto_shrink(&mut self) {
        if let Some(ref aof_file) = self.aof_file {
            let current_size = aof_file.size().unwrap_or(0);

            if current_size >= self.config.auto_shrink_min_size {
                let threshold = self.last_aof_size
                    + (self.last_aof_size * self.config.auto_shrink_percentage as u64 / 100);

                if current_size >= threshold {
                    // Perform actual shrinking
                    if !self.shrinking {
                        self.shrinking = true;

                        // Create a new temporary AOF file for compaction
                        if let Ok(mut shrink_file) = aof_file.create_shrink_file() {
                            // Write all current live data to the shrink file
                            for (key, item) in &self.keys {
                                // Skip expired items
                                if let Some(expires_at) = item.expires_at {
                                    if expires_at <= std::time::SystemTime::now() {
                                        continue;
                                    }
                                }

                                // Write the current key-value pair
                                let opts =
                                    item.expires_at.map(|expires_at| crate::types::SetOptions {
                                        expires_at: Some(expires_at),
                                        ttl: None,
                                    });

                                let _ =
                                    shrink_file.write_set(&key.clone(), &item.value, opts.as_ref());
                            }

                            // Flush and replace
                            let _ = shrink_file.flush();
                            if let Some(aof_file) = self.aof_file.as_mut() {
                                if aof_file.replace_with_shrink().is_ok() {
                                    self.last_aof_size = aof_file.size().unwrap_or(0);
                                }
                            }
                        }

                        self.shrinking = false;
                    }
                }
            }
        }
    }

    /// Insert an item into the database
    pub fn insert_item(&mut self, key: Bytes, item: DbItem) -> Option<DbItem> {
        // Remove from old expiration index if updating
        let old_item = if let Some(old) = self.keys.get(&key) {
            if let Some(expires_at) = old.expires_at {
                if let Some(keys_at_time) = self.expirations.get_mut(&expires_at) {
                    keys_at_time.retain(|k| k != &key);
                    if keys_at_time.is_empty() {
                        self.expirations.remove(&expires_at);
                    }
                }
            }
            Some(old.clone())
        } else {
            None
        };

        // Add to expiration index
        if let Some(expires_at) = item.expires_at {
            self.expirations
                .entry(expires_at)
                .or_default()
                .push(key.clone());
        }

        // Insert into main storage
        let result = self.keys.insert(key.clone(), item.clone());

        // Update indexes
        if let Some(ref old) = old_item {
            self.index_manager.remove_item(&key, &old.value);
        }
        self.index_manager.insert_item(&key, &item.value);

        // Update stats
        if old_item.is_none() {
            self.stats.key_count += 1;
        }

        result
    }

    /// Remove an item from the database
    pub fn remove_item(&mut self, key: &Bytes) -> Option<DbItem> {
        if let Some(item) = self.keys.remove(key) {
            // Remove from expiration index
            if let Some(expires_at) = item.expires_at {
                if let Some(keys_at_time) = self.expirations.get_mut(&expires_at) {
                    keys_at_time.retain(|k| k != key);
                    if keys_at_time.is_empty() {
                        self.expirations.remove(&expires_at);
                    }
                }
            }

            // Remove from indexes
            self.index_manager.remove_item(key, &item.value);

            // Update stats
            self.stats.key_count -= 1;

            Some(item)
        } else {
            None
        }
    }

    /// Get an item from the database
    pub fn get_item(&self, key: &Bytes) -> Option<&DbItem> {
        self.keys.get(key)
    }

    /// Check if the database contains a key
    #[allow(dead_code)]
    pub fn contains_key(&self, key: &Bytes) -> bool {
        self.keys.contains_key(key)
    }

    /// Get the number of keys in the database
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if the database is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl DB {
    // Spatial database operations
    /// Insert a point with automatic spatial indexing
    pub fn insert_point(
        &self,
        key: impl AsRef<[u8]>,
        point: &Point,
        opts: Option<SetOptions>,
    ) -> Result<Option<Bytes>> {
        let value = format!("{},{}", point.lat, point.lon);
        self.insert(key, value.as_bytes(), opts)
    }

    /// Insert a point with geohash indexing
    pub fn insert_point_with_geohash(
        &self,
        prefix: &str,
        point: &Point,
        precision: usize,
        data: impl AsRef<[u8]>,
        opts: Option<SetOptions>,
    ) -> Result<()> {
        let geohash = point.to_geohash(precision)?;
        let spatial_key = SpatialKey::geohash(prefix, &geohash);
        self.insert(spatial_key, data, opts)?;
        Ok(())
    }

    /// Insert a point with S2 cell indexing
    pub fn insert_point_with_s2(
        &self,
        prefix: &str,
        point: &Point,
        level: u8,
        data: impl AsRef<[u8]>,
        opts: Option<SetOptions>,
    ) -> Result<()> {
        let cell_id = point.to_s2_cell(level)?;
        let spatial_key = SpatialKey::s2_cell(prefix, cell_id);
        self.insert(spatial_key, data, opts)?;
        Ok(())
    }

    /// Find nearest neighbors within a radius
    pub fn find_nearest_neighbors(
        &self,
        prefix: &str,
        center: &Point,
        radius_meters: f64,
        limit: usize,
    ) -> Result<Vec<(String, Bytes, Point, f64)>> {
        let mut results = Vec::new();
        let inner = self.read()?;

        // Search for both simple point storage and geohash-indexed storage
        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if !item.is_expired() {
                let mut point_opt: Option<Point> = None;

                // Check if this is a geohash-indexed key
                if key_str.starts_with(&format!("{}:gh:", prefix)) {
                    // Extract geohash from key like "prefix:gh:geohash"
                    if let Some(geohash_part) = key_str.split(':').nth(2) {
                        if let Ok(decoded) = geohash::decode(geohash_part) {
                            point_opt = Some(Point::new(decoded.0.y, decoded.0.x));
                        }
                    }
                } else if key_str.starts_with(prefix) {
                    // Try to parse stored point data for simple storage
                    let value_str = String::from_utf8_lossy(&item.value);
                    if let Some((lat_str, lon_str)) = value_str.split_once(',') {
                        if let (Ok(lat), Ok(lon)) = (lat_str.parse::<f64>(), lon_str.parse::<f64>())
                        {
                            point_opt = Some(Point::new(lat, lon));
                        }
                    }
                }

                if let Some(point) = point_opt {
                    let distance = center.distance_to(&point);
                    if distance <= radius_meters {
                        results.push((key_str.to_string(), item.value.clone(), point, distance));
                    }
                }
            }
        }

        // Sort by distance and limit results
        results.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    /// Insert trajectory data
    pub fn insert_trajectory(
        &self,
        object_id: &str,
        points: &[(Point, u64)], // Point with timestamp
        opts: Option<SetOptions>,
    ) -> Result<()> {
        self.atomic(|batch| {
            for (point, timestamp) in points {
                let key = format!("{}:{}:{}", object_id, timestamp, point.to_geohash(12)?);
                let value = format!("{},{},{}", point.lat, point.lon, timestamp);
                batch.insert(&key, value.as_bytes(), opts.clone())?;
            }
            Ok(())
        })
    }

    /// Query trajectory between timestamps
    pub fn query_trajectory(
        &self,
        object_id: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<(Point, u64)>> {
        let mut results = Vec::new();
        let inner = self.read()?;
        let prefix = format!("{}:", object_id);

        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if key_str.starts_with(&prefix) && !item.is_expired() {
                let value_str = String::from_utf8_lossy(&item.value);
                let parts: Vec<&str> = value_str.split(',').collect();
                if parts.len() >= 3 {
                    if let (Ok(lat), Ok(lon), Ok(timestamp)) = (
                        parts[0].parse::<f64>(),
                        parts[1].parse::<f64>(),
                        parts[2].parse::<u64>(),
                    ) {
                        if timestamp >= start_time && timestamp <= end_time {
                            results.push((Point::new(lat, lon), timestamp));
                        }
                    }
                }
            }
        }

        results.sort_by_key(|(_, timestamp)| *timestamp);
        Ok(results)
    }

    /// Spatial statistics
    pub fn spatial_stats(&self) -> Result<SpatialStats> {
        let inner = self.read()?;
        let mut geohash_indexes = HashMap::new();
        let mut s2_indexes = HashMap::new();
        let mut total_points = 0;

        for key in inner.keys.keys() {
            let key_str = String::from_utf8_lossy(key);
            if key_str.contains(":gh:") {
                total_points += 1;
                if let Some(hash_part) = key_str.split(":gh:").nth(1) {
                    let precision = hash_part.len();
                    *geohash_indexes.entry(precision).or_insert(0) += 1;
                }
            } else if key_str.contains(":s2:") {
                total_points += 1;
                let level = 16; // Default S2 level
                *s2_indexes.entry(level).or_insert(0) += 1;
            }
        }

        Ok(SpatialStats {
            total_points,
            geohash_indexes,
            s2_indexes,
            grid_indexes: 1, // Simple count
        })
    }

    /// Simple spatial query methods
    pub fn intersects(
        &self,
        prefix: &str,
        query_point: &Point,
        radius_meters: f64,
    ) -> Result<Vec<(String, Bytes, Point, f64)>> {
        self.find_nearest_neighbors(prefix, query_point, radius_meters, usize::MAX)
    }

    pub fn nearby(
        &self,
        prefix: &str,
        center: &Point,
        radius_meters: f64,
        limit: usize,
    ) -> Result<Vec<(String, Bytes, Point, f64)>> {
        self.find_nearest_neighbors(prefix, center, radius_meters, limit)
    }

    pub fn within(&self, prefix: &str, bbox: &BoundingBox) -> Result<Vec<(String, Bytes, Point)>> {
        let mut results = Vec::new();
        let inner = self.read()?;

        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if !item.is_expired() {
                let mut point_opt: Option<Point> = None;

                // Check if this is a geohash-indexed key
                if key_str.starts_with(&format!("{}:gh:", prefix)) {
                    // Extract geohash from key like "prefix:gh:geohash"
                    if let Some(geohash_part) = key_str.split(':').nth(2) {
                        if let Ok(decoded) = geohash::decode(geohash_part) {
                            point_opt = Some(Point::new(decoded.0.y, decoded.0.x));
                        }
                    }
                } else if key_str.starts_with(prefix) {
                    // Try to parse stored point data for simple storage
                    let value_str = String::from_utf8_lossy(&item.value);
                    if let Some((lat_str, lon_str)) = value_str.split_once(',') {
                        if let (Ok(lat), Ok(lon)) = (lat_str.parse::<f64>(), lon_str.parse::<f64>())
                        {
                            point_opt = Some(Point::new(lat, lon));
                        }
                    }
                }

                if let Some(point) = point_opt {
                    if point.within_bounds(bbox.min.lat, bbox.min.lon, bbox.max.lat, bbox.max.lon) {
                        results.push((key_str.to_string(), item.value.clone(), point));
                    }
                }
            }
        }
        Ok(results)
    }

    /// Manually trigger cleanup of expired items (useful for testing)
    pub fn cleanup_expired(&self) -> Result<()> {
        let mut inner = self.write()?;
        inner.cleanup_expired();
        Ok(())
    }

    // ===== GEOMETRY OPERATIONS =====

    /// Insert a geometry object into the database
    pub fn insert_geometry(
        &self,
        key: impl AsRef<str>,
        geometry: &Geometry,
        opts: Option<SetOptions>,
    ) -> Result<()> {
        let bytes = geometry.to_bytes()?;
        self.insert(key.as_ref().as_bytes(), &bytes, opts.clone())?;
        Ok(())
    }

    /// Get a geometry object from the database
    pub fn get_geometry(&self, key: impl AsRef<str>) -> Result<Option<Geometry>> {
        if let Some(bytes) = self.get(key.as_ref().as_bytes())? {
            let geometry = Geometry::from_bytes(&bytes)?;
            Ok(Some(geometry))
        } else {
            Ok(None)
        }
    }

    /// Insert a polygon with spatial indexing
    pub fn insert_polygon(
        &self,
        namespace: impl AsRef<str>,
        polygon: &Polygon,
        value: &[u8],
        opts: Option<SetOptions>,
    ) -> Result<()> {
        let key = format!("{}:polygon:{}", namespace.as_ref(), uuid::Uuid::new_v4());

        // Store the polygon geometry
        let geometry = Geometry::Polygon(polygon.clone());
        self.insert_geometry(&key, &geometry, opts.clone())?;

        // Store associated value if provided
        if !value.is_empty() {
            let value_key = format!("{}:value", key);
            self.insert(value_key, value, opts.clone())?;
        }

        // Add spatial indexing based on bounds
        if let Some((min_coord, max_coord)) = polygon.bounds() {
            let center = Coordinate::new(
                (min_coord.x + max_coord.x) / 2.0,
                (min_coord.y + max_coord.y) / 2.0,
            );
            let center_point = center.to_point();

            // Use geohash indexing for the polygon center
            self.insert_point_with_geohash(
                &format!("{}:spatial", namespace.as_ref()),
                &center_point,
                8,
                key.as_bytes(),
                opts,
            )?;
        }

        Ok(())
    }

    /// Insert a linestring with spatial indexing
    pub fn insert_linestring(
        &self,
        namespace: impl AsRef<str>,
        linestring: &LineString,
        value: &[u8],
        opts: Option<SetOptions>,
    ) -> Result<()> {
        let key = format!("{}:linestring:{}", namespace.as_ref(), uuid::Uuid::new_v4());

        // Store the linestring geometry
        let geometry = Geometry::LineString(linestring.clone());
        self.insert_geometry(&key, &geometry, opts.clone())?;

        // Store associated value if provided
        if !value.is_empty() {
            let value_key = format!("{}:value", key);
            self.insert(value_key, value, opts.clone())?;
        }

        // Add spatial indexing for start and end points
        if let (Some(start), Some(end)) = (linestring.start_point(), linestring.end_point()) {
            let start_point = start.to_point();
            let end_point = end.to_point();

            self.insert_point_with_geohash(
                &format!("{}:spatial", namespace.as_ref()),
                &start_point,
                8,
                format!("{}:start", key).as_bytes(),
                opts.clone(),
            )?;

            self.insert_point_with_geohash(
                &format!("{}:spatial", namespace.as_ref()),
                &end_point,
                8,
                format!("{}:end", key).as_bytes(),
                opts,
            )?;
        }

        Ok(())
    }

    /// Query geometries that intersect with a given geometry
    pub fn intersects_geometry(
        &self,
        namespace: impl AsRef<str>,
        query_geometry: &Geometry,
    ) -> Result<Vec<(String, Geometry)>> {
        let inner = self.read()?;
        let mut results = Vec::new();
        let pattern = format!("{}:polygon:", namespace.as_ref());
        let pattern2 = format!("{}:linestring:", namespace.as_ref());

        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if key_str.starts_with(&pattern) || key_str.starts_with(&pattern2) {
                if let Ok(stored_geometry) = Geometry::from_bytes(&item.value) {
                    if GeometryOps::intersects(query_geometry, &stored_geometry) {
                        results.push((key_str.to_string(), stored_geometry));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Query geometries within a bounding box
    pub fn geometries_within_bounds(
        &self,
        namespace: impl AsRef<str>,
        min_coord: &Coordinate,
        max_coord: &Coordinate,
    ) -> Result<Vec<(String, Geometry)>> {
        let bbox_polygon =
            GeometryOps::rectangle(min_coord.x, min_coord.y, max_coord.x, max_coord.y)?;
        let query_geometry = Geometry::Polygon(bbox_polygon);
        self.intersects_geometry(namespace, &query_geometry)
    }

    /// Query geometries that contain a specific point
    pub fn geometries_containing_point(
        &self,
        namespace: impl AsRef<str>,
        point: &Coordinate,
    ) -> Result<Vec<(String, Geometry)>> {
        let inner = self.read()?;
        let mut results = Vec::new();
        let pattern = format!("{}:polygon:", namespace.as_ref());

        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if key_str.starts_with(&pattern) {
                if let Ok(geometry) = Geometry::from_bytes(&item.value) {
                    if geometry.contains_point(point) {
                        results.push((key_str.to_string(), geometry));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Calculate the distance between a point and the nearest geometry
    pub fn nearest_geometry_distance(
        &self,
        namespace: impl AsRef<str>,
        point: &Coordinate,
    ) -> Result<Option<(String, Geometry, f64)>> {
        let inner = self.read()?;
        let mut min_distance = f64::INFINITY;
        let mut nearest_geometry = None;
        let pattern = format!("{}:", namespace.as_ref());
        let query_point = Geometry::Point(point.clone());

        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if key_str.starts_with(&pattern)
                && (key_str.contains(":polygon:") || key_str.contains(":linestring:"))
            {
                if let Ok(geometry) = Geometry::from_bytes(&item.value) {
                    let distance = GeometryOps::distance(&query_point, &geometry);
                    if distance < min_distance {
                        min_distance = distance;
                        nearest_geometry = Some((key_str.to_string(), geometry));
                    }
                }
            }
        }

        if let Some((key, geometry)) = nearest_geometry {
            Ok(Some((key, geometry, min_distance)))
        } else {
            Ok(None)
        }
    }

    /// Get all geometries in a namespace with their metadata
    pub fn list_geometries(
        &self,
        namespace: impl AsRef<str>,
    ) -> Result<Vec<(String, Geometry, Option<Bytes>)>> {
        let inner = self.read()?;
        let mut results = Vec::new();
        let pattern = format!("{}:", namespace.as_ref());

        for (key, item) in &inner.keys {
            let key_str = String::from_utf8_lossy(key);
            if key_str.starts_with(&pattern)
                && (key_str.contains(":polygon:") || key_str.contains(":linestring:"))
            {
                if let Ok(geometry) = Geometry::from_bytes(&item.value) {
                    // Try to get associated value
                    let value_key = format!("{}:value", key_str);
                    let value = inner
                        .keys
                        .get(value_key.as_bytes())
                        .map(|item| item.value.clone());
                    results.push((key_str.to_string(), geometry, value));
                }
            }
        }

        Ok(results)
    }

    /// Calculate total area of all polygons in a namespace
    pub fn total_polygon_area(&self, namespace: impl AsRef<str>) -> Result<f64> {
        let geometries = self.list_geometries(namespace)?;
        let total_area = geometries
            .iter()
            .map(|(_, geometry, _)| geometry.area())
            .sum();
        Ok(total_area)
    }

    /// Calculate total length of all linestrings in a namespace
    pub fn total_linestring_length(&self, namespace: impl AsRef<str>) -> Result<f64> {
        let geometries = self.list_geometries(namespace)?;
        let total_length = geometries
            .iter()
            .map(|(_, geometry, _)| geometry.length())
            .sum();
        Ok(total_length)
    }
}

/// Spatial statistics structure
#[derive(Debug)]
pub struct SpatialStats {
    pub total_points: usize,
    pub geohash_indexes: HashMap<usize, usize>,
    pub s2_indexes: HashMap<u8, usize>,
    pub grid_indexes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_db_creation() {
        let db = DB::memory().unwrap();
        assert!(db.stats().unwrap().key_count == 0);
    }

    #[test]
    fn test_basic_operations() {
        let db = DB::memory().unwrap();

        // Single atomic insert
        db.insert("key1", &b"value1"[..], None).unwrap();

        // Get value
        let value = db.get("key1").unwrap().unwrap();
        assert_eq!(value, &b"value1"[..]);
    }

    #[test]
    fn test_atomic_batch() {
        let db = DB::memory().unwrap();

        // Atomic batch of operations
        db.atomic(|batch| {
            batch.insert("key1", &b"value1"[..], None)?;
            batch.insert("key2", &b"value2"[..], None)?;
            Ok(())
        })
        .unwrap();

        assert_eq!(db.get("key1").unwrap().unwrap(), &b"value1"[..]);
        assert_eq!(db.get("key2").unwrap().unwrap(), &b"value2"[..]);
    }

    #[test]
    fn test_expiration() {
        let db = DB::memory().unwrap();

        let opts = SetOptions::with_ttl(Duration::from_millis(100));
        db.insert("key1", &b"value1"[..], Some(opts)).unwrap();

        // Should exist initially
        assert_eq!(db.get("key1").unwrap().unwrap(), &b"value1"[..]);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired now
        assert!(db.get("key1").unwrap().is_none());
    }

    #[test]
    fn test_aof_persistence_and_replay() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        // Create database with persistence
        {
            let db = DB::open(db_path).unwrap();

            // Insert some data
            db.insert("key1", &b"value1"[..], None).unwrap();
            db.insert("key2", &b"value2"[..], None).unwrap();

            let opts = SetOptions::with_ttl(Duration::from_secs(3600));
            db.insert("key3", &b"value3"[..], Some(opts)).unwrap();

            // Delete a key
            db.delete("key2").unwrap();

            // Force AOF sync
            let inner = db.read().unwrap();
            if let Some(ref aof_file) = inner.aof_file {
                let mut aof_clone = AOFFile::open(aof_file.path()).unwrap();
                aof_clone.sync().unwrap();
            }
        }

        // Reopen database - should replay from AOF
        {
            let db = DB::open(db_path).unwrap();

            // Verify data was restored
            assert_eq!(db.get("key1").unwrap().unwrap(), &b"value1"[..]);
            assert!(db.get("key2").unwrap().is_none()); // Was deleted
            assert_eq!(db.get("key3").unwrap().unwrap(), &b"value3"[..]);

            // Verify stats
            let stats = db.stats().unwrap();
            assert_eq!(stats.key_count, 2); // key1 and key3
        }
    }

    #[test]
    fn test_aof_shrink() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        let db = DB::open(db_path).unwrap();

        // Insert some data
        db.insert("key1", &b"value1"[..], None).unwrap();
        db.insert("key2", &b"value2"[..], None).unwrap();
        db.insert("key3", &b"value3"[..], None).unwrap();

        // Delete some data to create "dead" entries in AOF
        db.delete("key2").unwrap();

        // Force sync to ensure data is written
        db.sync().unwrap();

        // Get initial AOF size
        let initial_size = {
            let inner = db.read().unwrap();
            inner.aof_file.as_ref().unwrap().size().unwrap()
        };

        // Perform shrink
        db.shrink().unwrap();

        // Get size after shrink
        let final_size = {
            let inner = db.read().unwrap();
            inner.aof_file.as_ref().unwrap().size().unwrap()
        };

        // AOF should be smaller after shrinking (removed deleted key2)
        assert!(final_size < initial_size);

        // Verify data integrity after shrink
        assert_eq!(db.get("key1").unwrap().unwrap(), &b"value1"[..]);
        assert!(db.get("key2").unwrap().is_none()); // Still deleted
        assert_eq!(db.get("key3").unwrap().unwrap(), &b"value3"[..]);
    }

    #[test]
    fn test_aof_shrink_with_expiration() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        let db = DB::open(db_path).unwrap();

        // Insert data with short expiration
        let opts = SetOptions::with_ttl(Duration::from_millis(50));
        db.insert("expired_key", &b"expired_value"[..], Some(opts))
            .unwrap();

        // Insert normal data
        db.insert("normal_key", &b"normal_value"[..], None).unwrap();

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(100));

        // Perform shrink - should remove expired entries
        db.shrink().unwrap();

        // Verify expired key is gone and normal key remains
        assert!(db.get("expired_key").unwrap().is_none());
        assert_eq!(db.get("normal_key").unwrap().unwrap(), &b"normal_value"[..]);
    }

    #[test]
    fn test_auto_shrink_trigger() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        let db = DB::open(db_path).unwrap();

        // Set a small auto-shrink threshold for testing
        let mut config = db.config().unwrap();
        config.auto_shrink_min_size = 100; // Very small threshold
        config.auto_shrink_percentage = 50; // Trigger at 50% growth
        db.set_config(config).unwrap();

        // Insert enough data to trigger auto-shrink
        for i in 0..100 {
            db.insert(format!("key{}", i), &b"some_value_here"[..], None)
                .unwrap();
        }

        // Delete half the data to create opportunities for shrinking
        for i in 0..50 {
            db.delete(format!("key{}", i)).unwrap();
        }

        // Force sync to update AOF
        db.sync().unwrap();

        // Simulate background task that would trigger auto-shrink
        {
            let mut inner = db.write().unwrap();
            inner.maybe_auto_shrink();
        }

        // Verify remaining data is still accessible
        for i in 50..100 {
            assert_eq!(
                db.get(format!("key{}", i)).unwrap().unwrap(),
                &b"some_value_here"[..]
            );
        }

        // Verify deleted data is gone
        for i in 0..50 {
            assert!(db.get(format!("key{}", i)).unwrap().is_none());
        }
    }

    #[test]
    fn test_shrink_empty_database() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        let db = DB::open(db_path).unwrap();

        // Shrink empty database should not fail
        db.shrink().unwrap();

        // Should still be able to use database
        db.insert("test_key", &b"test_value"[..], None).unwrap();
        assert_eq!(db.get("test_key").unwrap().unwrap(), &b"test_value"[..]);
    }

    #[test]
    fn test_shrink_memory_database() {
        let db = DB::memory().unwrap();

        // Insert some data
        db.insert("key1", &b"value1"[..], None).unwrap();

        // Shrink memory database should not fail (no AOF file)
        db.shrink().unwrap();

        // Data should still be accessible
        assert_eq!(db.get("key1").unwrap().unwrap(), &b"value1"[..]);
    }

    #[test]
    fn test_geometry_operations() {
        use crate::geometry::{Coordinate, Polygon};

        let db = DB::memory().unwrap();

        // Test polygon insertion and retrieval
        let coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(1.0, 0.0),
            Coordinate::new(1.0, 1.0),
            Coordinate::new(0.0, 1.0),
            Coordinate::new(0.0, 0.0),
        ];
        let ring = crate::geometry::LinearRing::new(coords).unwrap();
        let polygon = Polygon::new(ring);

        db.insert_polygon("test", &polygon, b"test polygon", None)
            .unwrap();

        // Test point containment query
        let test_point = Coordinate::new(0.5, 0.5);
        let containing_geometries = db.geometries_containing_point("test", &test_point).unwrap();
        assert!(!containing_geometries.is_empty());

        // Test bounding box query
        let min_coord = Coordinate::new(-1.0, -1.0);
        let max_coord = Coordinate::new(2.0, 2.0);
        let geometries_in_bounds = db
            .geometries_within_bounds("test", &min_coord, &max_coord)
            .unwrap();
        assert!(!geometries_in_bounds.is_empty());
    }

    #[test]
    fn test_linestring_operations() {
        use crate::geometry::{Coordinate, LineString};

        let db = DB::memory().unwrap();

        // Test linestring insertion
        let coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(1.0, 1.0),
            Coordinate::new(2.0, 0.0),
        ];
        let linestring = LineString::new(coords).unwrap();

        db.insert_linestring("routes", &linestring, b"route 1", None)
            .unwrap();

        // Test geometry listing
        let geometries = db.list_geometries("routes").unwrap();
        assert_eq!(geometries.len(), 1);

        // Test total length calculation
        let total_length = db.total_linestring_length("routes").unwrap();
        assert!(total_length > 0.0);
    }

    #[test]
    fn test_geometry_serialization() {
        use crate::geometry::{Coordinate, Geometry};

        let db = DB::memory().unwrap();
        let point = Coordinate::new(1.0, 2.0);
        let geometry = Geometry::Point(point);

        db.insert_geometry("test_geom", &geometry, None).unwrap();
        let retrieved = db.get_geometry("test_geom").unwrap().unwrap();

        assert_eq!(geometry, retrieved);
    }

    #[test]
    fn test_nearest_geometry() {
        use crate::geometry::{Coordinate, GeometryOps};

        let db = DB::memory().unwrap();

        // Insert a rectangle
        let rect = GeometryOps::rectangle(0.0, 0.0, 1.0, 1.0).unwrap();
        db.insert_polygon("shapes", &rect, b"rectangle", None)
            .unwrap();

        // Find nearest geometry to a point
        let query_point = Coordinate::new(2.0, 2.0);
        let nearest = db
            .nearest_geometry_distance("shapes", &query_point)
            .unwrap();

        assert!(nearest.is_some());
        let (_, _, distance) = nearest.unwrap();
        assert!(distance > 0.0);
    }
}
