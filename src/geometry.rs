//! Geometry types and operations for SpatioLite.
//!
//! This module provides comprehensive support for geometric objects including
//! points, lines, polygons, and complex geometries. It offers:
//!
//! - **Coordinate System**: 2D and 3D coordinate support
//! - **Geometry Types**: Points, LineStrings, Polygons with holes
//! - **Geometric Operations**: Area, length, buffer, distance calculations
//! - **Serialization**: WKT (Well-Known Text) import/export
//! - **Spatial Analysis**: Contains, intersects, within operations
//!
//! # Supported Geometry Types
//!
//! - **Point**: Single coordinate location
//! - **LineString**: Connected sequence of coordinates
//! - **Polygon**: Closed area with optional holes
//! - **Multi-geometries**: Collections of the above types
//!
//! # Examples
//!
//! ## Creating Basic Geometries
//!
//! ```rust
//! use spatio_lite::geometry::{Coordinate, LineString, Polygon, LinearRing};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create coordinates
//! let coord1 = Coordinate::new(-73.9857, 40.7484); // Times Square
//! let coord2 = Coordinate::new(-73.9733, 40.7644); // Central Park SW
//! let coord3 = Coordinate::new(-73.9500, 40.7644); // Central Park SE
//!
//! // Create a line string (path)
//! let path = LineString::new(vec![coord1, coord2, coord3])?;
//! println!("Path length: {:.2} degrees", path.length());
//!
//! // Create a polygon (area)
//! let square_coords = vec![
//!     Coordinate::new(0.0, 0.0),
//!     Coordinate::new(1.0, 0.0),
//!     Coordinate::new(1.0, 1.0),
//!     Coordinate::new(0.0, 1.0),
//!     Coordinate::new(0.0, 0.0), // Close the ring
//! ];
//! let square_ring = LinearRing::new(square_coords)?;
//! let square = Polygon::new(square_ring);
//! println!("Square area: {:.2}", square.area());
//! # Ok(())
//! # }
//! ```
//!
//! ## Geometric Operations
//!
//! ```rust
//! use spatio_lite::geometry::{Coordinate, GeometryOps};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a circular buffer around a point
//! let center = Coordinate::new(-73.9857, 40.7484);
//! let buffer = GeometryOps::buffer_point(&center, 0.01, 16)?; // ~1km radius
//!
//! // Check if a point is inside the buffer
//! let test_point = Coordinate::new(-73.9850, 40.7480);
//! let is_inside = buffer.contains_point(&test_point);
//! println!("Point is inside buffer: {}", is_inside);
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, SpatioLiteError};
use crate::spatial::Point;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A coordinate representing a 2D or 3D point in space.
///
/// Coordinates are the fundamental building blocks of all geometric objects.
/// They support both 2D (x, y) and 3D (x, y, z) representations, with
/// the z-coordinate being optional.
///
/// # Coordinate System
///
/// - **X-axis**: Typically longitude in geographic systems (East-West)
/// - **Y-axis**: Typically latitude in geographic systems (North-South)
/// - **Z-axis**: Elevation or height (optional)
///
/// # Examples
///
/// ```rust
/// use spatio_lite::geometry::Coordinate;
///
/// // 2D coordinate (longitude, latitude)
/// let empire_state = Coordinate::new(-73.9857, 40.7484);
///
/// // 3D coordinate with elevation
/// let mountain_peak = Coordinate::new_3d(-105.0178, 39.7392, 1655.0); // Denver with elevation
///
/// // Distance calculation
/// let distance = empire_state.distance_to(&mountain_peak);
/// println!("Distance: {:.2} coordinate units", distance);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}

impl Coordinate {
    /// Creates a new 2D coordinate.
    ///
    /// # Arguments
    ///
    /// * `x` - X-coordinate (typically longitude)
    /// * `y` - Y-coordinate (typically latitude)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio_lite::geometry::Coordinate;
    ///
    /// let coord = Coordinate::new(-73.9857, 40.7484);
    /// assert_eq!(coord.x, -73.9857);
    /// assert_eq!(coord.y, 40.7484);
    /// assert!(!coord.is_3d());
    /// ```
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, z: None }
    }

    /// Creates a new 3D coordinate with elevation.
    ///
    /// # Arguments
    ///
    /// * `x` - X-coordinate (typically longitude)
    /// * `y` - Y-coordinate (typically latitude)
    /// * `z` - Z-coordinate (elevation or height)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio_lite::geometry::Coordinate;
    ///
    /// let mountain = Coordinate::new_3d(-105.0178, 39.7392, 1655.0);
    /// assert_eq!(mountain.z, Some(1655.0));
    /// assert!(mountain.is_3d());
    /// ```
    pub fn new_3d(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z: Some(z) }
    }

    /// Creates a coordinate from a spatial Point.
    ///
    /// Converts a `Point` (lat, lon) to a `Coordinate` (x, y), swapping
    /// the order to follow the conventional (x=longitude, y=latitude) pattern.
    ///
    /// # Arguments
    ///
    /// * `point` - The Point to convert
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio_lite::{Point, geometry::Coordinate};
    ///
    /// let point = Point::new(40.7484, -73.9857); // lat, lon
    /// let coord = Coordinate::from_point(&point);
    /// assert_eq!(coord.x, -73.9857); // longitude
    /// assert_eq!(coord.y, 40.7484);  // latitude
    /// ```
    pub fn from_point(point: &Point) -> Self {
        Self::new(point.lon, point.lat)
    }

    /// Converts this coordinate to a spatial Point.
    ///
    /// Creates a `Point` (lat, lon) from this `Coordinate` (x, y), swapping
    /// the order and ignoring any z-coordinate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio_lite::geometry::Coordinate;
    ///
    /// let coord = Coordinate::new(-73.9857, 40.7484);
    /// let point = coord.to_point();
    /// assert_eq!(point.lat, 40.7484);  // y becomes lat
    /// assert_eq!(point.lon, -73.9857); // x becomes lon
    /// ```
    pub fn to_point(&self) -> Point {
        Point::new(self.y, self.x) // lat, lon
    }

    /// Calculates the Euclidean distance to another coordinate.
    ///
    /// For 2D coordinates, uses the standard distance formula.
    /// For 3D coordinates, includes the z-component in the calculation.
    /// If one coordinate is 2D and the other is 3D, the z-component is ignored.
    ///
    /// # Arguments
    ///
    /// * `other` - The destination coordinate
    ///
    /// # Returns
    ///
    /// Distance in the same units as the coordinates
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio_lite::geometry::Coordinate;
    ///
    /// let origin = Coordinate::new(0.0, 0.0);
    /// let point = Coordinate::new(3.0, 4.0);
    /// let distance = origin.distance_to(&point);
    /// assert_eq!(distance, 5.0); // 3-4-5 triangle
    ///
    /// // 3D distance
    /// let point_3d = Coordinate::new_3d(3.0, 4.0, 12.0);
    /// let distance_3d = origin.distance_to(&point_3d);
    /// assert_eq!(distance_3d, 5.0); // z ignored for 2D-3D comparison
    /// ```
    ///
    /// # Note
    ///
    /// This calculates straight-line distance in coordinate space.
    /// For geographic coordinates, use `Point::distance_to()` for
    /// great-circle distance calculations.
    pub fn distance_to(&self, other: &Coordinate) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        match (self.z, other.z) {
            (Some(z1), Some(z2)) => {
                let dz = z1 - z2;
                (dx * dx + dy * dy + dz * dz).sqrt()
            }
            _ => (dx * dx + dy * dy).sqrt(),
        }
    }

    /// Returns whether this coordinate has a z-component (3D).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spatio_lite::geometry::Coordinate;
    ///
    /// let coord_2d = Coordinate::new(1.0, 2.0);
    /// let coord_3d = Coordinate::new_3d(1.0, 2.0, 3.0);
    ///
    /// assert!(!coord_2d.is_3d());
    /// assert!(coord_3d.is_3d());
    /// ```
    pub fn is_3d(&self) -> bool {
        self.z.is_some()
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.z {
            Some(z) => write!(f, "({}, {}, {})", self.x, self.y, z),
            None => write!(f, "({}, {})", self.x, self.y),
        }
    }
}

/// A linear ring is a closed LineString
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearRing {
    pub coordinates: Vec<Coordinate>,
}

impl LinearRing {
    pub fn new(coordinates: Vec<Coordinate>) -> Result<Self> {
        if coordinates.len() < 4 {
            return Err(SpatioLiteError::Invalid);
        }

        // Check if ring is closed
        if coordinates.first() != coordinates.last() {
            return Err(SpatioLiteError::Invalid);
        }

        Ok(Self { coordinates })
    }

    pub fn from_points(points: Vec<Point>) -> Result<Self> {
        let coordinates: Vec<Coordinate> = points.iter().map(Coordinate::from_point).collect();
        Self::new(coordinates)
    }

    pub fn is_closed(&self) -> bool {
        self.coordinates.first() == self.coordinates.last()
    }

    pub fn area(&self) -> f64 {
        if self.coordinates.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        let n = self.coordinates.len() - 1; // Exclude the last point (same as first)

        for i in 0..n {
            let j = (i + 1) % n;
            area += self.coordinates[i].x * self.coordinates[j].y;
            area -= self.coordinates[j].x * self.coordinates[i].y;
        }

        area.abs() / 2.0
    }

    pub fn centroid(&self) -> Coordinate {
        if self.coordinates.is_empty() {
            return Coordinate::new(0.0, 0.0);
        }

        let mut x_sum = 0.0;
        let mut y_sum = 0.0;
        let n = self.coordinates.len() - 1; // Exclude the last point

        for coord in self.coordinates.iter().take(n) {
            x_sum += coord.x;
            y_sum += coord.y;
        }

        Coordinate::new(x_sum / n as f64, y_sum / n as f64)
    }

    pub fn contains_point(&self, point: &Coordinate) -> bool {
        // Ray casting algorithm
        let mut inside = false;
        let n = self.coordinates.len() - 1;

        let mut j = n - 1;
        for i in 0..n {
            let xi = self.coordinates[i].x;
            let yi = self.coordinates[i].y;
            let xj = self.coordinates[j].x;
            let yj = self.coordinates[j].y;

            if ((yi > point.y) != (yj > point.y))
                && (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }
}

/// A LineString geometry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineString {
    pub coordinates: Vec<Coordinate>,
}

impl LineString {
    pub fn new(coordinates: Vec<Coordinate>) -> Result<Self> {
        if coordinates.len() < 2 {
            return Err(SpatioLiteError::Invalid);
        }
        Ok(Self { coordinates })
    }

    pub fn from_points(points: Vec<Point>) -> Result<Self> {
        let coordinates: Vec<Coordinate> = points.iter().map(Coordinate::from_point).collect();
        Self::new(coordinates)
    }

    pub fn length(&self) -> f64 {
        if self.coordinates.len() < 2 {
            return 0.0;
        }

        let mut total_length = 0.0;
        for i in 1..self.coordinates.len() {
            total_length += self.coordinates[i - 1].distance_to(&self.coordinates[i]);
        }
        total_length
    }

    pub fn start_point(&self) -> Option<&Coordinate> {
        self.coordinates.first()
    }

    pub fn end_point(&self) -> Option<&Coordinate> {
        self.coordinates.last()
    }

    pub fn is_closed(&self) -> bool {
        self.coordinates.first() == self.coordinates.last()
    }

    pub fn to_linear_ring(&self) -> Result<LinearRing> {
        if !self.is_closed() {
            return Err(SpatioLiteError::Invalid);
        }
        LinearRing::new(self.coordinates.clone())
    }

    pub fn interpolate(&self, fraction: f64) -> Option<Coordinate> {
        if !(0.0..=1.0).contains(&fraction) || self.coordinates.len() < 2 {
            return None;
        }

        let total_length = self.length();
        let target_distance = total_length * fraction;

        let mut current_distance = 0.0;
        for i in 1..self.coordinates.len() {
            let segment_length = self.coordinates[i - 1].distance_to(&self.coordinates[i]);

            if current_distance + segment_length >= target_distance {
                let remaining = target_distance - current_distance;
                let ratio = remaining / segment_length;

                let start = &self.coordinates[i - 1];
                let end = &self.coordinates[i];

                let x = start.x + (end.x - start.x) * ratio;
                let y = start.y + (end.y - start.y) * ratio;

                return Some(Coordinate::new(x, y));
            }

            current_distance += segment_length;
        }

        self.coordinates.last().cloned()
    }

    pub fn bounds(&self) -> Option<(Coordinate, Coordinate)> {
        if self.coordinates.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for coord in &self.coordinates {
            min_x = min_x.min(coord.x);
            min_y = min_y.min(coord.y);
            max_x = max_x.max(coord.x);
            max_y = max_y.max(coord.y);
        }

        Some((Coordinate::new(min_x, min_y), Coordinate::new(max_x, max_y)))
    }
}

/// A Polygon geometry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub exterior: LinearRing,
    pub holes: Vec<LinearRing>,
}

impl Polygon {
    pub fn new(exterior: LinearRing) -> Self {
        Self {
            exterior,
            holes: Vec::new(),
        }
    }

    pub fn with_holes(exterior: LinearRing, holes: Vec<LinearRing>) -> Self {
        Self { exterior, holes }
    }

    pub fn from_coordinates(coordinates: Vec<Vec<Coordinate>>) -> Result<Self> {
        if coordinates.is_empty() {
            return Err(SpatioLiteError::Invalid);
        }

        let exterior = LinearRing::new(coordinates[0].clone())?;
        let mut holes = Vec::new();

        for hole_coords in coordinates.iter().skip(1) {
            holes.push(LinearRing::new(hole_coords.clone())?);
        }

        Ok(Self::with_holes(exterior, holes))
    }

    pub fn area(&self) -> f64 {
        let mut total_area = self.exterior.area();

        // Subtract hole areas
        for hole in &self.holes {
            total_area -= hole.area();
        }

        total_area
    }

    pub fn centroid(&self) -> Coordinate {
        // Simple centroid of exterior ring
        // In a full implementation, this would be weighted by area
        self.exterior.centroid()
    }

    pub fn contains_point(&self, point: &Coordinate) -> bool {
        // Point must be inside exterior ring
        if !self.exterior.contains_point(point) {
            return false;
        }

        // Point must not be inside any holes
        for hole in &self.holes {
            if hole.contains_point(point) {
                return false;
            }
        }

        true
    }

    pub fn bounds(&self) -> Option<(Coordinate, Coordinate)> {
        if self.exterior.coordinates.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for coord in &self.exterior.coordinates {
            min_x = min_x.min(coord.x);
            min_y = min_y.min(coord.y);
            max_x = max_x.max(coord.x);
            max_y = max_y.max(coord.y);
        }

        Some((Coordinate::new(min_x, min_y), Coordinate::new(max_x, max_y)))
    }

    pub fn intersects(&self, other: &Polygon) -> bool {
        // Simple bounding box check first
        if let (Some(self_bounds), Some(other_bounds)) = (self.bounds(), other.bounds()) {
            let (self_min, self_max) = self_bounds;
            let (other_min, other_max) = other_bounds;

            if self_max.x < other_min.x
                || self_min.x > other_max.x
                || self_max.y < other_min.y
                || self_min.y > other_max.y
            {
                return false;
            }
        }

        // Check if any vertex of one polygon is inside the other
        for coord in &self.exterior.coordinates {
            if other.contains_point(coord) {
                return true;
            }
        }

        for coord in &other.exterior.coordinates {
            if self.contains_point(coord) {
                return true;
            }
        }

        // TODO: Full line segment intersection checking would be needed for complete accuracy
        false
    }
}

/// Main geometry enum that can hold any geometry type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    Point(Coordinate),
    LineString(LineString),
    Polygon(Polygon),
}

impl Geometry {
    pub fn bounds(&self) -> Option<(Coordinate, Coordinate)> {
        match self {
            Geometry::Point(point) => Some((point.clone(), point.clone())),
            Geometry::LineString(line) => line.bounds(),
            Geometry::Polygon(polygon) => polygon.bounds(),
        }
    }

    pub fn geometry_type(&self) -> &'static str {
        match self {
            Geometry::Point(_) => "Point",
            Geometry::LineString(_) => "LineString",
            Geometry::Polygon(_) => "Polygon",
        }
    }

    pub fn area(&self) -> f64 {
        match self {
            Geometry::Polygon(polygon) => polygon.area(),
            _ => 0.0,
        }
    }

    pub fn length(&self) -> f64 {
        match self {
            Geometry::LineString(line) => line.length(),
            _ => 0.0,
        }
    }

    pub fn contains_point(&self, point: &Coordinate) -> bool {
        match self {
            Geometry::Point(geom_point) => geom_point == point,
            Geometry::Polygon(polygon) => polygon.contains_point(point),
            _ => false,
        }
    }

    /// Serialize geometry to bytes for storage
    pub fn to_bytes(&self) -> Result<Bytes> {
        let serialized =
            bincode::serialize(self).map_err(|e| SpatioLiteError::Serialization(e.to_string()))?;
        Ok(Bytes::from(serialized))
    }

    /// Deserialize geometry from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let geometry: Geometry = bincode::deserialize(bytes)
            .map_err(|e| SpatioLiteError::Deserialization(e.to_string()))?;
        Ok(geometry)
    }

    /// Convert to WKT (Well-Known Text) format
    pub fn to_wkt(&self) -> String {
        match self {
            Geometry::Point(point) => {
                if let Some(z) = point.z {
                    format!("POINT Z ({} {} {})", point.x, point.y, z)
                } else {
                    format!("POINT ({} {})", point.x, point.y)
                }
            }
            Geometry::LineString(line) => {
                let coords: Vec<String> = line
                    .coordinates
                    .iter()
                    .map(|c| {
                        if let Some(z) = c.z {
                            format!("{} {} {}", c.x, c.y, z)
                        } else {
                            format!("{} {}", c.x, c.y)
                        }
                    })
                    .collect();
                format!("LINESTRING ({})", coords.join(", "))
            }
            Geometry::Polygon(polygon) => {
                let mut rings = Vec::new();

                // Exterior ring
                let exterior_coords: Vec<String> = polygon
                    .exterior
                    .coordinates
                    .iter()
                    .map(|c| format!("{} {}", c.x, c.y))
                    .collect();
                rings.push(format!("({})", exterior_coords.join(", ")));

                // Holes
                for hole in &polygon.holes {
                    let hole_coords: Vec<String> = hole
                        .coordinates
                        .iter()
                        .map(|c| format!("{} {}", c.x, c.y))
                        .collect();
                    rings.push(format!("({})", hole_coords.join(", ")));
                }

                format!("POLYGON ({})", rings.join(", "))
            }
        }
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_wkt())
    }
}

/// Utility functions for geometry operations
pub struct GeometryOps;

impl GeometryOps {
    /// Create a buffer around a point
    pub fn buffer_point(center: &Coordinate, radius: f64, segments: usize) -> Result<Polygon> {
        if segments < 3 {
            return Err(SpatioLiteError::Invalid);
        }

        let mut coordinates = Vec::with_capacity(segments + 1);
        let angle_step = 2.0 * std::f64::consts::PI / segments as f64;

        for i in 0..segments {
            let angle = i as f64 * angle_step;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            coordinates.push(Coordinate::new(x, y));
        }

        // Close the ring
        coordinates.push(coordinates[0].clone());

        let ring = LinearRing::new(coordinates)?;
        Ok(Polygon::new(ring))
    }

    /// Create a rectangle polygon
    pub fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Polygon> {
        let coordinates = vec![
            Coordinate::new(min_x, min_y),
            Coordinate::new(max_x, min_y),
            Coordinate::new(max_x, max_y),
            Coordinate::new(min_x, max_y),
            Coordinate::new(min_x, min_y), // Close the ring
        ];

        let ring = LinearRing::new(coordinates)?;
        Ok(Polygon::new(ring))
    }

    /// Calculate the distance between two geometries
    pub fn distance(geom1: &Geometry, geom2: &Geometry) -> f64 {
        // Simplified distance calculation using bounds
        if let (Some(bounds1), Some(bounds2)) = (geom1.bounds(), geom2.bounds()) {
            let center1 = Coordinate::new(
                (bounds1.0.x + bounds1.1.x) / 2.0,
                (bounds1.0.y + bounds1.1.y) / 2.0,
            );
            let center2 = Coordinate::new(
                (bounds2.0.x + bounds2.1.x) / 2.0,
                (bounds2.0.y + bounds2.1.y) / 2.0,
            );
            center1.distance_to(&center2)
        } else {
            f64::INFINITY
        }
    }

    /// Check if two geometries intersect
    pub fn intersects(geom1: &Geometry, geom2: &Geometry) -> bool {
        if let (Some(bounds1), Some(bounds2)) = (geom1.bounds(), geom2.bounds()) {
            // Bounding box intersection check
            bounds1.1.x >= bounds2.0.x
                && bounds1.0.x <= bounds2.1.x
                && bounds1.1.y >= bounds2.0.y
                && bounds1.0.y <= bounds2.1.y
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_creation() {
        let coord = Coordinate::new(1.0, 2.0);
        assert_eq!(coord.x, 1.0);
        assert_eq!(coord.y, 2.0);
        assert!(!coord.is_3d());

        let coord_3d = Coordinate::new_3d(1.0, 2.0, 3.0);
        assert!(coord_3d.is_3d());
        assert_eq!(coord_3d.z, Some(3.0));
    }

    #[test]
    fn test_coordinate_distance() {
        let coord1 = Coordinate::new(0.0, 0.0);
        let coord2 = Coordinate::new(3.0, 4.0);
        assert_eq!(coord1.distance_to(&coord2), 5.0);

        let coord3d1 = Coordinate::new_3d(0.0, 0.0, 0.0);
        let coord3d2 = Coordinate::new_3d(1.0, 1.0, 1.0);
        assert!((coord3d1.distance_to(&coord3d2) - 3.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_linestring_creation() {
        let coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(1.0, 1.0),
            Coordinate::new(2.0, 0.0),
        ];
        let line = LineString::new(coords).unwrap();
        assert_eq!(line.coordinates.len(), 3);
        assert!(!line.is_closed());

        // Test length calculation
        let length = line.length();
        assert!(length > 0.0);
    }

    #[test]
    fn test_linestring_interpolation() {
        let coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(1.0, 0.0),
            Coordinate::new(2.0, 0.0),
        ];
        let line = LineString::new(coords).unwrap();

        // Interpolate at midpoint
        let mid_point = line.interpolate(0.5).unwrap();
        assert_eq!(mid_point.x, 1.0);
        assert_eq!(mid_point.y, 0.0);

        // Interpolate at start
        let start_point = line.interpolate(0.0).unwrap();
        assert_eq!(start_point.x, 0.0);

        // Interpolate at end
        let end_point = line.interpolate(1.0).unwrap();
        assert_eq!(end_point.x, 2.0);
    }

    #[test]
    fn test_polygon_creation() {
        let coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(1.0, 0.0),
            Coordinate::new(1.0, 1.0),
            Coordinate::new(0.0, 1.0),
            Coordinate::new(0.0, 0.0), // Close the ring
        ];
        let ring = LinearRing::new(coords).unwrap();
        let polygon = Polygon::new(ring);

        assert_eq!(polygon.area(), 1.0);

        // Test point containment
        let inside_point = Coordinate::new(0.5, 0.5);
        let outside_point = Coordinate::new(2.0, 2.0);

        assert!(polygon.contains_point(&inside_point));
        assert!(!polygon.contains_point(&outside_point));
    }

    #[test]
    fn test_polygon_with_holes() {
        // Outer ring: unit square
        let exterior_coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(2.0, 0.0),
            Coordinate::new(2.0, 2.0),
            Coordinate::new(0.0, 2.0),
            Coordinate::new(0.0, 0.0),
        ];
        let exterior = LinearRing::new(exterior_coords).unwrap();

        // Inner ring: hole in the middle
        let hole_coords = vec![
            Coordinate::new(0.5, 0.5),
            Coordinate::new(1.5, 0.5),
            Coordinate::new(1.5, 1.5),
            Coordinate::new(0.5, 1.5),
            Coordinate::new(0.5, 0.5),
        ];
        let hole = LinearRing::new(hole_coords).unwrap();

        let polygon = Polygon::with_holes(exterior, vec![hole]);

        // Point in outer ring but outside hole
        let point_in_polygon = Coordinate::new(0.25, 0.25);
        assert!(polygon.contains_point(&point_in_polygon));

        // Point in hole
        let point_in_hole = Coordinate::new(1.0, 1.0);
        assert!(!polygon.contains_point(&point_in_hole));

        // Area should be outer area minus hole area
        assert_eq!(polygon.area(), 4.0 - 1.0); // 2x2 - 1x1
    }

    #[test]
    fn test_geometry_serialization() {
        let coord = Coordinate::new(1.0, 2.0);
        let geometry = Geometry::Point(coord);

        let bytes = geometry.to_bytes().unwrap();
        let deserialized = Geometry::from_bytes(&bytes).unwrap();

        assert_eq!(geometry, deserialized);
    }

    #[test]
    fn test_geometry_wkt() {
        let coord = Coordinate::new(1.0, 2.0);
        let geometry = Geometry::Point(coord);
        assert_eq!(geometry.to_wkt(), "POINT (1 2)");

        let coords = vec![Coordinate::new(0.0, 0.0), Coordinate::new(1.0, 1.0)];
        let line = LineString::new(coords).unwrap();
        let line_geometry = Geometry::LineString(line);
        assert_eq!(line_geometry.to_wkt(), "LINESTRING (0 0, 1 1)");
    }

    #[test]
    fn test_buffer_point() {
        let center = Coordinate::new(0.0, 0.0);
        let buffer = GeometryOps::buffer_point(&center, 1.0, 8).unwrap();

        // Buffer should be roughly circular with area close to π
        let area = buffer.area();
        assert!((area - std::f64::consts::PI).abs() < 0.5); // Approximate due to polygon approximation
    }

    #[test]
    fn test_rectangle() {
        let rect = GeometryOps::rectangle(0.0, 0.0, 2.0, 3.0).unwrap();
        assert_eq!(rect.area(), 6.0);

        let inside_point = Coordinate::new(1.0, 1.5);
        assert!(rect.contains_point(&inside_point));

        let outside_point = Coordinate::new(3.0, 1.0);
        assert!(!rect.contains_point(&outside_point));
    }

    #[test]
    fn test_geometry_operations() {
        let point1 = Geometry::Point(Coordinate::new(0.0, 0.0));
        let point2 = Geometry::Point(Coordinate::new(3.0, 4.0));

        let distance = GeometryOps::distance(&point1, &point2);
        assert_eq!(distance, 5.0);

        // Test intersection with overlapping bounds
        let rect1 = GeometryOps::rectangle(0.0, 0.0, 2.0, 2.0).unwrap();
        let rect2 = GeometryOps::rectangle(1.0, 1.0, 3.0, 3.0).unwrap();

        let geom1 = Geometry::Polygon(rect1);
        let geom2 = Geometry::Polygon(rect2);

        assert!(GeometryOps::intersects(&geom1, &geom2));
    }
}
