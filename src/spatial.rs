//! Spatial utilities for Spatio
//!
//! This module provides core spatial functionality for geographic points
//! and basic spatial operations.

use crate::error::{Result, SpatioError};
use geohash;
use s2::cellid::CellID;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A geographic point representing a location on Earth's surface.
///
/// `Point` stores latitude and longitude coordinates and provides methods
/// for spatial operations, distance calculations, and spatial indexing.
/// All coordinates use the WGS84 coordinate reference system (EPSG:4326).
///
/// # Examples
///
/// ```rust
/// use spatio::Point;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Major world cities
/// let new_york = Point::new(40.7128, -74.0060);
/// let london = Point::new(51.5074, -0.1278);
///
/// // Calculate distance between cities
/// let distance_km = new_york.distance_to(&london) / 1000.0;
/// println!("NYC to London: {:.0} km", distance_km);
///
/// // Generate spatial index keys
/// let geohash = new_york.to_geohash(8)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Latitude in decimal degrees (-90.0 to +90.0)
    pub lat: f64,
    /// Longitude in decimal degrees (-180.0 to +180.0)
    pub lon: f64,
}

impl Point {
    /// Creates a new point from latitude and longitude coordinates.
    ///
    /// # Arguments
    ///
    /// * `lat` - Latitude in decimal degrees (-90° to +90°)
    /// * `lon` - Longitude in decimal degrees (-180° to +180°)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::Point;
    ///
    /// // Empire State Building
    /// let empire_state = Point::new(40.7484, -73.9857);
    ///
    /// // Sydney Opera House
    /// let opera_house = Point::new(-33.8568, 151.2153);
    /// ```
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// Calculate the distance between two points using the Haversine formula.
    ///
    /// This method calculates the great-circle distance between two points
    /// on Earth's surface, accounting for Earth's curvature. The result
    /// is returned in meters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::Point;
    ///
    /// let new_york = Point::new(40.7128, -74.0060);
    /// let london = Point::new(51.5074, -0.1278);
    ///
    /// let distance_m = new_york.distance_to(&london);
    /// let distance_km = distance_m / 1000.0;
    /// println!("Distance: {:.0} km", distance_km);
    /// ```
    pub fn distance_to(&self, other: &Point) -> f64 {
        const EARTH_RADIUS_M: f64 = 6_371_000.0;

        let lat1_rad = self.lat.to_radians();
        let lat2_rad = other.lat.to_radians();
        let delta_lat = (other.lat - self.lat).to_radians();
        let delta_lon = (other.lon - self.lon).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_M * c
    }

    /// Generate a geohash string for this point.
    ///
    /// Geohash is a geocoding system that represents geographic coordinates
    /// as a short string. Higher precision values result in more precise
    /// location encoding but longer strings.
    ///
    /// # Arguments
    ///
    /// * `precision` - Number of characters in the geohash (1-12)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::Point;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let point = Point::new(40.7128, -74.0060); // NYC
    /// let geohash = point.to_geohash(8)?;
    /// println!("NYC geohash: {}", geohash); // e.g., "dr5regw3"
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_geohash(&self, precision: usize) -> Result<String> {
        geohash::encode(
            geo::Coord {
                x: self.lon,
                y: self.lat,
            },
            precision,
        )
        .map_err(|_| SpatioError::InvalidGeohash)
    }

    /// Generate an S2 cell ID for this point.
    ///
    /// S2 is Google's library for spherical geometry. It represents
    /// Earth's surface as a hierarchy of cells that can be used for
    /// spatial indexing and proximity queries.
    ///
    /// # Arguments
    ///
    /// * `level` - S2 cell level (0-30, higher = more precise)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::Point;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let point = Point::new(40.7128, -74.0060);
    /// let s2_cell = point.to_s2_cell(16)?;
    /// println!("S2 cell: {}", s2_cell.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_s2_cell(&self, level: u8) -> Result<CellID> {
        if level > 30 {
            return Err(SpatioError::Other("S2 level must be <= 30".to_string()));
        }

        // Simplified S2 cell ID generation
        // Map lat/lon to cell coordinates and combine with level
        let lat_norm = ((self.lat + 90.0) / 180.0 * ((1u64 << level) as f64)) as u64;
        let lon_norm = ((self.lon + 180.0) / 360.0 * ((1u64 << level) as f64)) as u64;

        let cell_value = (level as u64) << 56 | lat_norm << 28 | lon_norm;
        Ok(CellID(cell_value))
    }

    /// Check if this point is within the given bounding box.
    ///
    /// # Arguments
    ///
    /// * `min_lat` - Minimum latitude of bounding box
    /// * `min_lon` - Minimum longitude of bounding box
    /// * `max_lat` - Maximum latitude of bounding box
    /// * `max_lon` - Maximum longitude of bounding box
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::Point;
    ///
    /// let point = Point::new(40.7128, -74.0060); // NYC
    /// let in_usa = point.within_bounds(24.0, -125.0, 49.0, -66.0);
    /// assert!(in_usa);
    /// ```
    pub fn within_bounds(&self, min_lat: f64, min_lon: f64, max_lat: f64, max_lon: f64) -> bool {
        self.lat >= min_lat && self.lat <= max_lat && self.lon >= min_lon && self.lon <= max_lon
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.6}, {:.6})", self.lat, self.lon)
    }
}

/// Spatial key generation utilities for database storage.
///
/// This struct provides methods to generate keys for spatial indexing
/// based on different spatial indexing strategies.
pub struct SpatialKey;

impl SpatialKey {
    /// Generate a geohash-based key for database storage.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Namespace prefix for the key
    /// * `geohash` - The geohash string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::spatial::SpatialKey;
    ///
    /// let key = SpatialKey::geohash("cities", "dr5regw3");
    /// assert_eq!(key, "cities:gh:dr5regw3");
    /// ```
    pub fn geohash(prefix: &str, geohash: &str) -> String {
        format!("{}:gh:{}", prefix, geohash)
    }

    /// Generate an S2 cell-based key for database storage.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Namespace prefix for the key
    /// * `cell_id` - The S2 cell ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio::spatial::SpatialKey;
    /// use s2::cellid::CellID;
    ///
    /// let cell_id = CellID(1234567890);
    /// let key = SpatialKey::s2_cell("sensors", cell_id);
    /// assert_eq!(key, "sensors:s2:1234567890");
    /// ```
    pub fn s2_cell(prefix: &str, cell_id: CellID) -> String {
        format!("{}:s2:{}", prefix, cell_id.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(40.7128, -74.0060);
        assert_eq!(point.lat, 40.7128);
        assert_eq!(point.lon, -74.0060);
    }

    #[test]
    fn test_distance_calculation() {
        let new_york = Point::new(40.7128, -74.0060);
        let london = Point::new(51.5074, -0.1278);

        let distance = new_york.distance_to(&london);
        // Distance should be approximately 5585 km
        assert!((distance - 5_585_000.0).abs() < 50_000.0);
    }

    #[test]
    fn test_geohash_generation() {
        let point = Point::new(40.7128, -74.0060);
        let geohash = point.to_geohash(8).unwrap();
        assert_eq!(geohash.len(), 8);
    }

    #[test]
    fn test_s2_cell_generation() {
        let point = Point::new(40.7128, -74.0060);
        let s2_cell = point.to_s2_cell(16).unwrap();
        assert!(s2_cell.0 > 0);
    }

    #[test]
    fn test_within_bounds() {
        let point = Point::new(40.7128, -74.0060);

        // Should be within USA bounds
        assert!(point.within_bounds(24.0, -125.0, 49.0, -66.0));

        // Should not be within Europe bounds
        assert!(!point.within_bounds(35.0, -10.0, 70.0, 40.0));
    }

    #[test]
    fn test_spatial_key_generation() {
        let geohash_key = SpatialKey::geohash("cities", "dr5regw3");
        assert_eq!(geohash_key, "cities:gh:dr5regw3");

        let s2_key = SpatialKey::s2_cell("sensors", CellID(1234567890));
        assert_eq!(s2_key, "sensors:s2:1234567890");
    }

    #[test]
    fn test_point_display() {
        let point = Point::new(40.7128, -74.0060);
        let display = format!("{}", point);
        assert_eq!(display, "(40.712800, -74.006000)");
    }
}
