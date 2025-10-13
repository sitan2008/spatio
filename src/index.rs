use crate::error::{Result, SpatioError};
use crate::spatial::Point;
use bytes::Bytes;
use geohash;
use std::collections::HashMap;

/// Simplified index manager focused on spatial operations only.
///
/// This manages spatial indexes for efficient geographic queries.
/// It automatically handles geohash-based indexing for points.
pub struct IndexManager {
    /// Spatial indexes organized by prefix
    spatial_indexes: HashMap<String, SpatialIndex>,
}

/// A spatial index for a specific prefix/namespace
struct SpatialIndex {
    /// Points stored with their geohash keys
    points: HashMap<String, (Point, Bytes)>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        Self {
            spatial_indexes: HashMap::new(),
        }
    }

    /// Insert a point into the spatial index
    pub fn insert_point(&mut self, prefix: &str, point: &Point, data: &Bytes) -> Result<()> {
        let index = self
            .spatial_indexes
            .entry(prefix.to_string())
            .or_insert_with(SpatialIndex::new);

        let geohash = point
            .to_geohash(8)
            .map_err(|_| SpatioError::InvalidGeohash)?;

        index.points.insert(geohash, (*point, data.clone()));
        Ok(())
    }

    /// Find nearby points within a radius
    pub fn find_nearby(
        &self,
        prefix: &str,
        center: &Point,
        radius_meters: f64,
        limit: usize,
    ) -> Result<Vec<(Point, Bytes)>> {
        let index = match self.spatial_indexes.get(prefix) {
            Some(index) => index,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();

        // For large search radii or when geohash approach doesn't find enough points,
        // fall back to checking all points
        if radius_meters > 100_000.0 || index.points.len() < 1000 {
            // Check all points in the index
            for (point, data) in index.points.values() {
                let distance = center.distance_to(point);
                if distance <= radius_meters {
                    results.push((*point, data.clone()));
                }
            }
        } else {
            // Use geohash-based search for efficiency
            let mut candidates = std::collections::HashSet::new();

            // Try multiple precision levels for better coverage
            for precision in [6, 7, 8] {
                if let Ok(center_geohash) = center.to_geohash(precision) {
                    candidates.insert(center_geohash.clone());

                    // Add neighbors at this precision
                    for direction in &[
                        geohash::Direction::N,
                        geohash::Direction::S,
                        geohash::Direction::E,
                        geohash::Direction::W,
                        geohash::Direction::NE,
                        geohash::Direction::NW,
                        geohash::Direction::SE,
                        geohash::Direction::SW,
                    ] {
                        if let Ok(neighbor) = geohash::neighbor(&center_geohash, *direction) {
                            candidates.insert(neighbor);
                        }
                    }
                }
            }

            // Check all candidate geohashes
            for geohash in candidates {
                // Check if any point starts with this geohash prefix
                for (stored_geohash, (point, data)) in &index.points {
                    if stored_geohash.starts_with(&geohash) || geohash.starts_with(stored_geohash) {
                        let distance = center.distance_to(point);
                        if distance <= radius_meters {
                            results.push((*point, data.clone()));
                        }
                    }
                }
            }

            // If we didn't find enough results, fall back to full scan
            if results.is_empty() {
                for (point, data) in index.points.values() {
                    let distance = center.distance_to(point);
                    if distance <= radius_meters {
                        results.push((*point, data.clone()));
                    }
                }
            }
        }

        // Sort by distance and limit results
        results.sort_by(|a, b| {
            let dist_a = center.distance_to(&a.0);
            let dist_b = center.distance_to(&b.0);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(limit);
        Ok(results)
    }

    /// Find all points within a bounding box
    pub fn find_within_bounds(
        &self,
        prefix: &str,
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
        limit: usize,
    ) -> Result<Vec<(Point, Bytes)>> {
        let index = match self.spatial_indexes.get(prefix) {
            Some(index) => index,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();

        // Check all points in the index
        for (point, data) in index.points.values() {
            if point.within_bounds(min_lat, min_lon, max_lat, max_lon) {
                results.push((*point, data.clone()));
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Check if there are any points within a circular region
    pub fn contains_point(&self, prefix: &str, center: &Point, radius_meters: f64) -> Result<bool> {
        let index = match self.spatial_indexes.get(prefix) {
            Some(index) => index,
            None => return Ok(false),
        };

        // For small datasets or large radii, just check all points
        if radius_meters > 100_000.0 || index.points.len() < 1000 {
            for (point, _) in index.points.values() {
                if center.distance_to(point) <= radius_meters {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        // Use geohash-based search for efficiency
        let mut candidates = std::collections::HashSet::new();

        // Try multiple precision levels for better coverage
        for precision in [6, 7, 8] {
            if let Ok(center_geohash) = center.to_geohash(precision) {
                candidates.insert(center_geohash.clone());

                // Add neighbors at this precision
                for direction in &[
                    geohash::Direction::N,
                    geohash::Direction::S,
                    geohash::Direction::E,
                    geohash::Direction::W,
                    geohash::Direction::NE,
                    geohash::Direction::NW,
                    geohash::Direction::SE,
                    geohash::Direction::SW,
                ] {
                    if let Ok(neighbor) = geohash::neighbor(&center_geohash, *direction) {
                        candidates.insert(neighbor);
                    }
                }
            }
        }

        // Check all candidate geohashes
        for geohash in candidates {
            // Check if any point starts with this geohash prefix
            for (stored_geohash, (point, _)) in &index.points {
                if (stored_geohash.starts_with(&geohash) || geohash.starts_with(stored_geohash))
                    && center.distance_to(point) <= radius_meters
                {
                    return Ok(true);
                }
            }
        }

        // If geohash search didn't find anything, fall back to full scan
        for (point, _) in index.points.values() {
            if center.distance_to(point) <= radius_meters {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if there are any points within a bounding box
    pub fn intersects_bounds(
        &self,
        prefix: &str,
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
    ) -> Result<bool> {
        let index = match self.spatial_indexes.get(prefix) {
            Some(index) => index,
            None => return Ok(false),
        };

        // Check if any point intersects with the bounding box
        for (point, _) in index.points.values() {
            if point.within_bounds(min_lat, min_lon, max_lat, max_lon) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Count points within a distance from a center point
    pub fn count_within_distance(
        &self,
        prefix: &str,
        center: &Point,
        radius_meters: f64,
    ) -> Result<usize> {
        let index = match self.spatial_indexes.get(prefix) {
            Some(index) => index,
            None => return Ok(0),
        };

        let mut count = 0;

        // For small datasets or large radii, just check all points
        if radius_meters > 100_000.0 || index.points.len() < 1000 {
            for (point, _) in index.points.values() {
                if center.distance_to(point) <= radius_meters {
                    count += 1;
                }
            }
            return Ok(count);
        }

        // Use geohash-based search for efficiency
        let mut candidates = std::collections::HashSet::new();

        // Try multiple precision levels for better coverage
        for precision in [6, 7, 8] {
            if let Ok(center_geohash) = center.to_geohash(precision) {
                candidates.insert(center_geohash.clone());

                // Add neighbors at this precision
                for direction in &[
                    geohash::Direction::N,
                    geohash::Direction::S,
                    geohash::Direction::E,
                    geohash::Direction::W,
                    geohash::Direction::NE,
                    geohash::Direction::NW,
                    geohash::Direction::SE,
                    geohash::Direction::SW,
                ] {
                    if let Ok(neighbor) = geohash::neighbor(&center_geohash, *direction) {
                        candidates.insert(neighbor);
                    }
                }
            }
        }

        // Check all candidate geohashes
        let mut found_points = std::collections::HashSet::new();
        for geohash in candidates {
            // Check if any point starts with this geohash prefix
            for (stored_geohash, (point, _)) in &index.points {
                if (stored_geohash.starts_with(&geohash) || geohash.starts_with(stored_geohash))
                    && center.distance_to(point) <= radius_meters
                {
                    // Use point coordinates as key to avoid double counting
                    found_points.insert((point.lat.to_bits(), point.lon.to_bits()));
                }
            }
        }

        count = found_points.len();

        // If geohash search didn't find anything, fall back to full scan
        if count == 0 {
            for (point, _) in index.points.values() {
                if center.distance_to(point) <= radius_meters {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Remove a point from the spatial index
    pub fn remove_point(&mut self, prefix: &str, point: &Point) -> Result<()> {
        if let Some(index) = self.spatial_indexes.get_mut(prefix) {
            let geohash = point
                .to_geohash(8)
                .map_err(|_| SpatioError::InvalidGeohash)?;
            index.points.remove(&geohash);
        }
        Ok(())
    }

    /// Get statistics about spatial indexes
    pub fn stats(&self) -> IndexStats {
        let mut total_points = 0;
        let index_count = self.spatial_indexes.len();

        for index in self.spatial_indexes.values() {
            total_points += index.points.len();
        }

        IndexStats {
            index_count,
            total_points,
        }
    }
}

impl SpatialIndex {
    fn new() -> Self {
        Self {
            points: HashMap::new(),
        }
    }
}

/// Statistics about the index manager
#[derive(Debug)]
pub struct IndexStats {
    pub index_count: usize,
    pub total_points: usize,
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}
