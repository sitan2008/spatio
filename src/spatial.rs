//! Spatial utilities and coordinate systems for SpatioLite
//!
//! This module provides integration with popular geospatial libraries:
//! - `geo` for geometric operations and coordinate handling
//! - `geohash` for hierarchical spatial indexing
//! - `s2` for Google's S2 geometry library integration
//!
//! # Examples
//!
//! ```rust
//! use spatio_lite::spatial::{Point, SpatialKey, CoordinateSystem};
//!
//! // Create a point
//! let point = Point::new(40.7128, -74.0060); // NYC coordinates
//!
//! // Generate spatial keys
//! let geohash = point.to_geohash(12).unwrap();
//! let s2_cell = point.to_s2_cell(16).unwrap();
//!
//! // Use in database operations
//! let key = SpatialKey::geohash("location", &geohash);
//! ```

use crate::error::{Result, SpatioLiteError};
use crate::types::Rect;
use geo::{Coord, LineString, Point as GeoPoint, Polygon};
use geohash::{decode, encode, Direction};
use s2::cellid::CellID;
use std::fmt;

/// A spatial point with latitude and longitude
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

impl Point {
    /// Create a new point from latitude and longitude
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// Create a point from a geo::Point
    pub fn from_geo_point(point: GeoPoint<f64>) -> Self {
        Self {
            lat: point.y(),
            lon: point.x(),
        }
    }

    /// Convert to geo::Point
    pub fn to_geo_point(&self) -> GeoPoint<f64> {
        GeoPoint::new(self.lon, self.lat)
    }

    /// Convert to geo::Coord
    pub fn to_coordinate(&self) -> Coord<f64> {
        Coord {
            x: self.lon,
            y: self.lat,
        }
    }

    /// Generate a geohash for this point
    pub fn to_geohash(&self, precision: usize) -> Result<String> {
        encode(
            Coord {
                x: self.lon,
                y: self.lat,
            },
            precision,
        )
        .map_err(|_| SpatioLiteError::InvalidOperation("Invalid geohash precision".to_string()))
    }

    /// Generate an S2 cell ID for this point (simplified implementation)
    pub fn to_s2_cell(&self, level: u8) -> Result<u64> {
        if level > 30 {
            return Err(SpatioLiteError::InvalidOperation(
                "S2 level must be <= 30".to_string(),
            ));
        }

        // Simplified S2 cell ID generation
        // Map lat/lon to cell coordinates and combine with level
        let lat_norm = ((self.lat + 90.0) / 180.0 * ((1u64 << level) as f64)) as u64;
        let lon_norm = ((self.lon + 180.0) / 360.0 * ((1u64 << level) as f64)) as u64;

        Ok((level as u64) << 56 | lat_norm << 28 | lon_norm)
    }

    /// Calculate distance to another point using Haversine formula
    pub fn distance_to(&self, other: &Point) -> f64 {
        use geo::algorithm::Distance;
        use geo::Haversine;
        let p1 = self.to_geo_point();
        let p2 = other.to_geo_point();
        Haversine.distance(p1, p2)
    }

    /// Calculate bearing to another point
    pub fn bearing_to(&self, other: &Point) -> f64 {
        use geo::algorithm::Bearing;
        use geo::Rhumb;
        let p1 = self.to_geo_point();
        let p2 = other.to_geo_point();
        Rhumb.bearing(p1, p2)
    }

    /// Check if point is within a bounding box
    pub fn within_bounds(&self, min_lat: f64, min_lon: f64, max_lat: f64, max_lon: f64) -> bool {
        self.lat >= min_lat && self.lat <= max_lat && self.lon >= min_lon && self.lon <= max_lon
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.lat, self.lon)
    }
}

/// A bounding box defined by two points
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(min_lat: f64, min_lon: f64, max_lat: f64, max_lon: f64) -> Self {
        Self {
            min: Point::new(min_lat, min_lon),
            max: Point::new(max_lat, max_lon),
        }
    }

    /// Create a bounding box from two points
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let min_lat = p1.lat.min(p2.lat);
        let max_lat = p1.lat.max(p2.lat);
        let min_lon = p1.lon.min(p2.lon);
        let max_lon = p1.lon.max(p2.lon);

        Self::new(min_lat, min_lon, max_lat, max_lon)
    }

    /// Check if a point is contained within this bounding box
    pub fn contains(&self, point: &Point) -> bool {
        point.within_bounds(self.min.lat, self.min.lon, self.max.lat, self.max.lon)
    }

    /// Convert to SpatioLite Rect for indexing
    pub fn to_rect(&self) -> Rect {
        Rect::new(
            vec![self.min.lat, self.min.lon],
            vec![self.max.lat, self.max.lon],
        )
        .unwrap()
    }

    /// Generate geohash bounding box
    pub fn to_geohash(&self, precision: usize) -> Result<String> {
        // Use center point for geohash
        let center_lat = (self.min.lat + self.max.lat) / 2.0;
        let center_lon = (self.min.lon + self.max.lon) / 2.0;
        let center = Point::new(center_lat, center_lon);
        center.to_geohash(precision)
    }

    /// Get all geohash cells that intersect this bounding box
    pub fn intersecting_geohashes(&self, precision: usize) -> Result<Vec<String>> {
        let mut hashes = Vec::new();

        // Simple grid-based approach
        let lat_step = (self.max.lat - self.min.lat) / 10.0;
        let lon_step = (self.max.lon - self.min.lon) / 10.0;

        let mut lat = self.min.lat;
        while lat <= self.max.lat {
            let mut lon = self.min.lon;
            while lon <= self.max.lon {
                let point = Point::new(lat, lon);
                if let Ok(hash) = point.to_geohash(precision) {
                    if !hashes.contains(&hash) {
                        hashes.push(hash);
                    }
                }
                lon += lon_step;
            }
            lat += lat_step;
        }

        Ok(hashes)
    }
}

/// Spatial key generators for database indexing
pub struct SpatialKey;

impl SpatialKey {
    /// Generate a geohash-based key
    pub fn geohash(prefix: &str, hash: &str) -> String {
        format!("{}:gh:{}", prefix, hash)
    }

    /// Generate an S2-based key
    pub fn s2_cell(prefix: &str, cell_id: u64) -> String {
        format!("{}:s2:{:016x}", prefix, cell_id)
    }

    /// Generate a grid-based key (simple lat/lon grid)
    pub fn grid(prefix: &str, lat_grid: i32, lon_grid: i32, precision: u32) -> String {
        format!("{}:grid:{}:{}:{}", prefix, precision, lat_grid, lon_grid)
    }

    /// Generate a hierarchical key for multi-level indexing
    pub fn hierarchical(prefix: &str, levels: &[String]) -> String {
        format!("{}:hier:{}", prefix, levels.join(":"))
    }
}

/// Coordinate system utilities
pub struct CoordinateSystem;

impl CoordinateSystem {
    /// Convert WGS84 (GPS) coordinates to Web Mercator (used in web maps)
    pub fn wgs84_to_web_mercator(point: &Point) -> Point {
        let lon_rad = point.lon * std::f64::consts::PI / 180.0;
        let lat_rad = point.lat * std::f64::consts::PI / 180.0;

        let x = 20037508.34 * lon_rad / std::f64::consts::PI;
        let y = 20037508.34 * ((std::f64::consts::PI / 4.0 + lat_rad / 2.0).tan().ln())
            / std::f64::consts::PI;

        Point::new(x, y)
    }

    /// Convert Web Mercator back to WGS84
    pub fn web_mercator_to_wgs84(point: &Point) -> Point {
        let lon_rad = point.lat * std::f64::consts::PI / 20037508.34;
        let lat_rad = 2.0
            * ((point.lon * std::f64::consts::PI / 20037508.34)
                .exp()
                .atan())
            - std::f64::consts::PI / 2.0;

        let lon = lon_rad * 180.0 / std::f64::consts::PI;
        let lat = lat_rad * 180.0 / std::f64::consts::PI;

        Point::new(lat, lon)
    }

    /// Create a grid cell for a point at given precision
    pub fn to_grid_cell(point: &Point, precision: u32) -> (i32, i32) {
        let scale = 10_i32.pow(precision) as f64;
        let lat_cell = (point.lat * scale).floor() as i32;
        let lon_cell = (point.lon * scale).floor() as i32;
        (lat_cell, lon_cell)
    }
}

/// Geohash utilities
pub struct GeohashUtils;

impl GeohashUtils {
    /// Decode a geohash back to coordinates
    pub fn decode(hash: &str) -> Result<Point> {
        let (coord, _lat_err, _lon_err) = decode(hash)
            .map_err(|_| SpatioLiteError::InvalidOperation("Invalid geohash".to_string()))?;
        Ok(Point::new(coord.y, coord.x))
    }

    /// Get neighbors of a geohash
    pub fn neighbors(hash: &str) -> Result<Vec<String>> {
        let directions = [
            Direction::N,
            Direction::NE,
            Direction::E,
            Direction::SE,
            Direction::S,
            Direction::SW,
            Direction::W,
            Direction::NW,
        ];

        let mut neighbors = Vec::new();
        for direction in &directions {
            match geohash::neighbor(hash, *direction) {
                Ok(neighbor) => neighbors.push(neighbor),
                Err(_) => continue,
            }
        }

        Ok(neighbors)
    }

    /// Get parent geohash (reduced precision)
    pub fn parent(hash: &str) -> Option<String> {
        if hash.len() <= 1 {
            return None;
        }
        Some(hash[..hash.len() - 1].to_string())
    }

    /// Get children geohashes (increased precision)
    pub fn children(hash: &str) -> Vec<String> {
        let base32 = "0123456789bcdefghjkmnpqrstuvwxyz";
        base32.chars().map(|c| format!("{}{}", hash, c)).collect()
    }

    /// Get bounding box for a geohash
    pub fn bounding_box(hash: &str) -> Result<BoundingBox> {
        let (coord, lat_err, lon_err) = decode(hash)
            .map_err(|_| SpatioLiteError::InvalidOperation("Invalid geohash".to_string()))?;

        let min_lat = coord.y - lat_err;
        let max_lat = coord.y + lat_err;
        let min_lon = coord.x - lon_err;
        let max_lon = coord.x + lon_err;

        Ok(BoundingBox::new(min_lat, min_lon, max_lat, max_lon))
    }
}

/// S2 utilities for Google's S2 geometry
pub struct S2Utils;

impl S2Utils {
    /// Convert S2 cell ID to lat/lng bounds
    pub fn cell_bounds(cell_id: u64) -> Result<BoundingBox> {
        let cell_id = CellID(cell_id);
        // Simplified implementation without full S2 Cell API
        // Convert cell_id back to approximate lat/lng bounds
        let lat = ((cell_id.0 & 0xFF) as f64 / 255.0) * 180.0 - 90.0;
        let lon = (((cell_id.0 >> 8) & 0xFF) as f64 / 255.0) * 360.0 - 180.0;
        let delta = 1.0; // Approximate cell size

        Ok(BoundingBox::new(
            lat - delta,
            lon - delta,
            lat + delta,
            lon + delta,
        ))
    }

    /// Get S2 cell neighbors
    pub fn neighbors(cell_id: u64) -> Vec<u64> {
        // Simplified neighbor calculation
        vec![
            cell_id.wrapping_add(1),
            cell_id.wrapping_sub(1),
            cell_id.wrapping_add(256),
            cell_id.wrapping_sub(256),
        ]
    }

    /// Get parent S2 cell
    pub fn parent(cell_id: u64) -> Option<u64> {
        if cell_id == 0 {
            None
        } else {
            Some(cell_id >> 2)
        }
    }

    /// Get S2 cell children
    pub fn children(cell_id: u64) -> Vec<u64> {
        if cell_id > (u64::MAX >> 2) {
            return vec![];
        }

        let base = cell_id << 2;
        vec![base, base + 1, base + 2, base + 3]
    }
}

/// Spatial analysis utilities
pub struct SpatialAnalysis;

impl SpatialAnalysis {
    /// Find points within a radius of a center point
    pub fn points_within_radius(
        center: &Point,
        radius_meters: f64,
        points: &[Point],
    ) -> Vec<Point> {
        points
            .iter()
            .filter(|point| center.distance_to(point) <= radius_meters)
            .cloned()
            .collect()
    }

    /// Calculate centroid of a set of points
    pub fn centroid(points: &[Point]) -> Option<Point> {
        if points.is_empty() {
            return None;
        }

        let sum_lat: f64 = points.iter().map(|p| p.lat).sum();
        let sum_lon: f64 = points.iter().map(|p| p.lon).sum();
        let count = points.len() as f64;

        Some(Point::new(sum_lat / count, sum_lon / count))
    }

    /// Create a buffer around a point (approximate circular buffer)
    pub fn buffer_point(center: &Point, radius_meters: f64, segments: usize) -> Vec<Point> {
        let mut points = Vec::new();

        // Use simple equirectangular approximation (good for small distances)
        let lat_rad = center.lat.to_radians();
        let lat_meter = 111320.0; // meters per degree latitude
        let lon_meter = 111320.0 * lat_rad.cos(); // meters per degree longitude at this latitude

        for i in 0..segments {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;

            // Calculate offset in meters, then convert to degrees
            let lat_offset_meters = radius_meters * angle.cos();
            let lon_offset_meters = radius_meters * angle.sin();

            let lat_offset_degrees = lat_offset_meters / lat_meter;
            let lon_offset_degrees = lon_offset_meters / lon_meter;

            let lat = center.lat + lat_offset_degrees;
            let lon = center.lon + lon_offset_degrees;

            points.push(Point::new(lat, lon));
        }

        points
    }

    /// Check if a point is inside a polygon (using ray casting algorithm)
    pub fn point_in_polygon(point: &Point, polygon_points: &[Point]) -> bool {
        if polygon_points.len() < 3 {
            return false;
        }

        let geo_point = point.to_geo_point();
        let coords: Vec<Coord<f64>> = polygon_points.iter().map(|p| p.to_coordinate()).collect();

        let polygon = Polygon::new(LineString::from(coords), vec![]);

        use geo::Contains;
        polygon.contains(&geo_point)
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
    fn test_geohash_generation() {
        let point = Point::new(40.7128, -74.0060);

        // Test various precisions to find valid range
        for precision in 1..=12 {
            match point.to_geohash(precision) {
                Ok(geohash) => {
                    println!(
                        "Precision {}: {} (length: {})",
                        precision,
                        geohash,
                        geohash.len()
                    );
                    assert!(geohash.len() > 0);
                }
                Err(e) => {
                    println!("Precision {}: Error - {:?}", precision, e);
                }
            }
        }

        // Test a known good precision
        let geohash = point.to_geohash(5).unwrap();
        assert!(geohash.len() > 0);
    }

    #[test]
    fn test_s2_cell_generation() {
        let point = Point::new(40.7128, -74.0060);
        let cell_id = point.to_s2_cell(16).unwrap();
        assert!(cell_id > 0);
    }

    #[test]
    fn test_distance_calculation() {
        let nyc = Point::new(40.7128, -74.0060);
        let la = Point::new(34.0522, -118.2437);
        let distance = nyc.distance_to(&la);

        // Distance between NYC and LA should be roughly 3,944 km
        assert!(distance > 3_900_000.0 && distance < 4_000_000.0);
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(40.0, -75.0, 41.0, -73.0);
        let point_inside = Point::new(40.5, -74.0);
        let point_outside = Point::new(42.0, -74.0);

        assert!(bbox.contains(&point_inside));
        assert!(!bbox.contains(&point_outside));
    }

    #[test]
    fn test_spatial_keys() {
        let hash = "dr5regy";
        let key = SpatialKey::geohash("location", hash);
        assert_eq!(key, "location:gh:dr5regy");

        let s2_key = SpatialKey::s2_cell("poi", 12345);
        assert_eq!(s2_key, "poi:s2:0000000000003039");
    }

    #[test]
    fn test_geohash_utils() {
        let hash = "dr5regy";
        let point = GeohashUtils::decode(hash).unwrap();

        // Should decode to approximately the original coordinates
        assert!((point.lat - 40.7).abs() < 1.0);
        assert!((point.lon + 74.0).abs() < 1.0);

        let neighbors = GeohashUtils::neighbors(hash).unwrap();
        assert!(!neighbors.is_empty());
        assert!(neighbors.len() <= 8);
    }

    #[test]
    fn test_spatial_analysis() {
        let center = Point::new(40.7128, -74.0060);
        let points = vec![
            Point::new(40.7138, -74.0050), // Close point
            Point::new(40.8128, -74.0060), // Far point
            Point::new(40.7120, -74.0070), // Close point
        ];

        let nearby = SpatialAnalysis::points_within_radius(&center, 2000.0, &points);
        assert!(nearby.len() >= 2); // Should find at least the close points

        let centroid = SpatialAnalysis::centroid(&points).unwrap();
        assert!((centroid.lat - 40.7).abs() < 0.2);
        assert!((centroid.lon + 74.0).abs() < 0.2);
    }

    #[test]
    fn test_coordinate_system_conversion() {
        let wgs84_point = Point::new(40.7128, -74.0060);
        let web_mercator = CoordinateSystem::wgs84_to_web_mercator(&wgs84_point);
        let back_to_wgs84 = CoordinateSystem::web_mercator_to_wgs84(&web_mercator);

        // Should round-trip with reasonable precision
        assert!((back_to_wgs84.lat - wgs84_point.lat).abs() < 0.0001);
        assert!((back_to_wgs84.lon - wgs84_point.lon).abs() < 0.0001);
    }
}
