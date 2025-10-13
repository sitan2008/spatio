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

        // Get candidate geohashes around the center point
        let center_geohash = center
            .to_geohash(8)
            .map_err(|_| SpatioError::InvalidGeohash)?;

        // Check the center geohash and its neighbors
        let mut candidates = vec![center_geohash.clone()];

        // Add neighboring geohashes for broader search
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
                candidates.push(neighbor);
            }
        }

        // Check all candidates
        for geohash in candidates {
            if let Some((point, data)) = index.points.get(&geohash) {
                let distance = center.distance_to(point);
                if distance <= radius_meters {
                    results.push((*point, data.clone()));
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
