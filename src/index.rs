use crate::error::{Result, SpatioLiteError};
use crate::types::{IndexOptions, IndexType, LessFunc, Rect, RectFunc};
use bytes::Bytes;
use rstar::{RTree, RTreeObject, AABB};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// A wrapper for items in the R-tree
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialItem {
    pub key: Bytes,
    pub value: Bytes,
    pub rect: Rect,
}

impl RTreeObject for SpatialItem {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        // Convert our multi-dimensional rect to 2D AABB for rstar
        // For higher dimensions, we'll use the first two dimensions
        let min_x = self.rect.min.get(0).copied().unwrap_or(0.0);
        let min_y = self.rect.min.get(1).copied().unwrap_or(0.0);
        let max_x = self.rect.max.get(0).copied().unwrap_or(0.0);
        let max_y = self.rect.max.get(1).copied().unwrap_or(0.0);

        AABB::from_corners([min_x, min_y], [max_x, max_y])
    }
}

/// Individual index structure
pub struct Index {
    /// Name of the index
    pub name: String,
    /// Pattern for key matching (simple glob-like pattern)
    pub pattern: String,
    /// Index type (B-tree or R-tree)
    pub index_type: IndexType,
    /// Options for this index
    pub options: IndexOptions,
    /// B-tree storage (for ordered indexes)
    btree: Option<BTreeMap<Bytes, Vec<Bytes>>>,
    /// R-tree storage (for spatial indexes)
    rtree: Option<RTree<SpatialItem>>,
    /// Custom comparison function for B-tree
    less_func: Option<Arc<LessFunc>>,
    /// Rectangle extraction function for R-tree
    rect_func: Option<Arc<RectFunc>>,
}

impl Index {
    /// Create a new B-tree index
    pub fn new_btree(
        name: String,
        pattern: String,
        less_func: Option<LessFunc>,
        options: IndexOptions,
    ) -> Self {
        Self {
            name,
            pattern,
            index_type: IndexType::BTree,
            options,
            btree: Some(BTreeMap::new()),
            rtree: None,
            less_func: less_func.map(Arc::new),
            rect_func: None,
        }
    }

    /// Create a new R-tree index
    pub fn new_rtree(
        name: String,
        pattern: String,
        rect_func: RectFunc,
        options: IndexOptions,
    ) -> Self {
        Self {
            name,
            pattern,
            index_type: IndexType::RTree,
            options,
            btree: None,
            rtree: Some(RTree::new()),
            less_func: None,
            rect_func: Some(Arc::new(rect_func)),
        }
    }

    /// Check if a key matches this index's pattern
    pub fn matches_pattern(&self, key: &[u8]) -> bool {
        let key_str = String::from_utf8_lossy(key);
        pattern_match(&self.pattern, &key_str, self.options.case_insensitive)
    }

    /// Insert an item into this index
    pub fn insert(&mut self, key: &Bytes, value: &Bytes) -> Result<()> {
        if !self.matches_pattern(key) {
            return Ok(());
        }

        match &mut self.btree {
            Some(btree) => {
                // B-tree index
                btree
                    .entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push(value.clone());
            }
            None => {
                // R-tree index
                if let Some(rtree) = &mut self.rtree {
                    if let Some(ref rect_func) = self.rect_func {
                        match rect_func(value) {
                            Ok(rect) => {
                                let spatial_item = SpatialItem {
                                    key: key.clone(),
                                    value: value.clone(),
                                    rect,
                                };
                                rtree.insert(spatial_item);
                            }
                            Err(e) => {
                                // Log error but don't fail the operation
                                eprintln!("Failed to extract rectangle for spatial index: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Remove an item from this index
    pub fn remove(&mut self, key: &Bytes, value: &Bytes) -> Result<()> {
        if !self.matches_pattern(key) {
            return Ok(());
        }

        match &mut self.btree {
            Some(btree) => {
                // B-tree index
                if let Some(values) = btree.get_mut(key) {
                    values.retain(|v| v != value);
                    if values.is_empty() {
                        btree.remove(key);
                    }
                }
            }
            None => {
                // R-tree index
                if let Some(rtree) = &mut self.rtree {
                    if let Some(ref rect_func) = self.rect_func {
                        if let Ok(rect) = rect_func(value) {
                            let spatial_item = SpatialItem {
                                key: key.clone(),
                                value: value.clone(),
                                rect,
                            };
                            rtree.remove(&spatial_item);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Query the index with a range (for B-tree indexes)
    pub fn range_query(
        &self,
        start: Option<&Bytes>,
        end: Option<&Bytes>,
    ) -> Result<Vec<(Bytes, Bytes)>> {
        match &self.btree {
            Some(btree) => {
                let mut results = Vec::new();

                let range = match (start, end) {
                    (Some(start), Some(end)) => btree.range::<Bytes, _>(start..=end),
                    (Some(start), None) => btree.range::<Bytes, _>(start..),
                    (None, Some(end)) => btree.range::<Bytes, _>(..=end),
                    (None, None) => btree.range::<Bytes, _>(..),
                };

                for (key, values) in range {
                    for value in values {
                        results.push((key.clone(), value.clone()));
                    }
                }

                Ok(results)
            }
            None => Err(SpatioLiteError::InvalidOperation(
                "Range query not supported on spatial indexes".to_string(),
            )),
        }
    }

    /// Spatial intersection query (for R-tree indexes)
    pub fn intersects(&self, rect: &Rect) -> Result<Vec<(Bytes, Bytes)>> {
        match &self.rtree {
            Some(rtree) => {
                let mut results = Vec::new();

                // Convert our Rect to AABB for rstar
                let min_x = rect.min.get(0).copied().unwrap_or(0.0);
                let min_y = rect.min.get(1).copied().unwrap_or(0.0);
                let max_x = rect.max.get(0).copied().unwrap_or(0.0);
                let max_y = rect.max.get(1).copied().unwrap_or(0.0);

                let query_aabb = AABB::from_corners([min_x, min_y], [max_x, max_y]);

                for item in rtree.locate_in_envelope_intersecting(&query_aabb) {
                    results.push((item.key.clone(), item.value.clone()));
                }

                Ok(results)
            }
            None => Err(SpatioLiteError::InvalidOperation(
                "Spatial query not supported on B-tree indexes".to_string(),
            )),
        }
    }

    /// Nearest neighbor query (for R-tree indexes)
    pub fn nearest(&self, point: &[f64], max_results: usize) -> Result<Vec<(Bytes, Bytes, f64)>> {
        match &self.rtree {
            Some(rtree) => {
                let mut results = Vec::new();

                if point.len() >= 2 {
                    let query_point = [point[0], point[1]];

                    // Simple distance calculation for all items since we can't use Point trait
                    for item in rtree.iter() {
                        let envelope = item.envelope();
                        let center_x = (envelope.lower()[0] + envelope.upper()[0]) / 2.0;
                        let center_y = (envelope.lower()[1] + envelope.upper()[1]) / 2.0;

                        let dx = center_x - query_point[0];
                        let dy = center_y - query_point[1];
                        let distance = (dx * dx + dy * dy).sqrt();

                        results.push((item.key.clone(), item.value.clone(), distance));
                    }

                    // Sort by distance and take max_results
                    results
                        .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
                    results.truncate(max_results);
                }

                Ok(results)
            }
            None => Err(SpatioLiteError::InvalidOperation(
                "Nearest neighbor query not supported on B-tree indexes".to_string(),
            )),
        }
    }

    /// Get the number of items in this index
    pub fn len(&self) -> usize {
        match &self.btree {
            Some(btree) => btree.values().map(|v| v.len()).sum(),
            None => match &self.rtree {
                Some(rtree) => rtree.size(),
                None => 0,
            },
        }
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Index manager handles all indexes for a database
pub struct IndexManager {
    indexes: HashMap<String, Index>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
        }
    }

    /// Create a new B-tree index
    pub fn create_btree_index(
        &mut self,
        name: String,
        pattern: String,
        less_func: Option<LessFunc>,
        options: IndexOptions,
    ) -> Result<()> {
        if self.indexes.contains_key(&name) {
            return Err(SpatioLiteError::IndexExists(name));
        }

        let index = Index::new_btree(name.clone(), pattern, less_func, options);
        self.indexes.insert(name, index);

        Ok(())
    }

    /// Create a new R-tree index
    pub fn create_rtree_index(
        &mut self,
        name: String,
        pattern: String,
        rect_func: RectFunc,
        options: IndexOptions,
    ) -> Result<()> {
        if self.indexes.contains_key(&name) {
            return Err(SpatioLiteError::IndexExists(name));
        }

        let index = Index::new_rtree(name.clone(), pattern, rect_func, options);
        self.indexes.insert(name, index);

        Ok(())
    }

    /// Drop an index
    pub fn drop_index(&mut self, name: &str) -> Result<()> {
        if self.indexes.remove(name).is_some() {
            Ok(())
        } else {
            Err(SpatioLiteError::IndexNotFound(name.to_string()))
        }
    }

    /// Get an index by name
    pub fn get_index(&self, name: &str) -> Option<&Index> {
        self.indexes.get(name)
    }

    /// Get a mutable reference to an index by name
    pub fn get_index_mut(&mut self, name: &str) -> Option<&mut Index> {
        self.indexes.get_mut(name)
    }

    /// Insert an item into all matching indexes
    pub fn insert_item(&mut self, key: &Bytes, value: &Bytes) {
        for index in self.indexes.values_mut() {
            let _ = index.insert(key, value);
        }
    }

    /// Remove an item from all matching indexes
    pub fn remove_item(&mut self, key: &Bytes, value: &Bytes) {
        for index in self.indexes.values_mut() {
            let _ = index.remove(key, value);
        }
    }

    /// List all index names
    pub fn list_indexes(&self) -> Vec<String> {
        self.indexes.keys().cloned().collect()
    }

    /// Get the number of indexes
    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    /// Check if there are no indexes
    pub fn is_empty(&self) -> bool {
        self.indexes.is_empty()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple glob-like pattern matching
fn pattern_match(pattern: &str, text: &str, case_insensitive: bool) -> bool {
    let pattern = if case_insensitive {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };
    let text = if case_insensitive {
        text.to_lowercase()
    } else {
        text.to_string()
    };

    // Simple implementation - just check if pattern is "*" (match all) or exact match
    // TODO: Implement full glob pattern matching with wildcards
    if pattern == "*" {
        true
    } else {
        pattern == text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_manager_creation() {
        let manager = IndexManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_btree_index_creation() {
        let mut manager = IndexManager::new();

        manager
            .create_btree_index(
                "test_index".to_string(),
                "*".to_string(),
                None,
                IndexOptions::default(),
            )
            .unwrap();

        assert_eq!(manager.len(), 1);
        assert!(manager.get_index("test_index").is_some());
    }

    #[test]
    fn test_rtree_index_creation() {
        let mut manager = IndexManager::new();

        let rect_func =
            Box::new(|_value: &[u8]| -> Result<Rect> { Ok(Rect::point(vec![1.0, 2.0])) });

        manager
            .create_rtree_index(
                "spatial_index".to_string(),
                "*".to_string(),
                rect_func,
                IndexOptions::default(),
            )
            .unwrap();

        assert_eq!(manager.len(), 1);
        assert!(manager.get_index("spatial_index").is_some());
    }

    #[test]
    fn test_pattern_matching() {
        assert!(pattern_match("*", "anything", false));
        assert!(pattern_match("test", "test", false));
        assert!(!pattern_match("test", "TEST", false));
        assert!(pattern_match("test", "TEST", true));
    }

    #[test]
    fn test_spatial_item() {
        let rect = Rect::point(vec![1.0, 2.0]);
        let item = SpatialItem {
            key: Bytes::from("key1"),
            value: Bytes::from("value1"),
            rect,
        };

        let envelope = item.envelope();
        assert_eq!(envelope.lower(), [1.0, 2.0]);
        assert_eq!(envelope.upper(), [1.0, 2.0]);
    }
}
