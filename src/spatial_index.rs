use crate::geometry::Geometry;
use pgrx::prelude::*;
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

/// Bounding box type for spatial indexing
/// This represents a 2D rectangular bounding box with min/max x,y coordinates
#[derive(Debug, Clone, PartialEq, PostgresType, Serialize, Deserialize)]
#[pg_binary_protocol]
#[inoutfuncs]
#[serde(rename_all = "camelCase")]
pub struct BBox {
    #[serde(rename = "minX")]
    pub min_x: f64,
    #[serde(rename = "minY")]
    pub min_y: f64,
    #[serde(rename = "maxX")]
    pub max_x: f64,
    #[serde(rename = "maxY")]
    pub max_y: f64,
}

impl BBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        BBox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_geometry(geom: &Geometry) -> Self {
        let (min_x, min_y, max_x, max_y) = geom.bounding_box();
        BBox::new(min_x, min_y, max_x, max_y)
    }

    /// Check if two bounding boxes overlap
    pub fn overlaps(&self, other: &BBox) -> bool {
        !(self.max_x < other.min_x
            || other.max_x < self.min_x
            || self.max_y < other.min_y
            || other.max_y < self.min_y)
    }

    /// Check if this bbox contains another
    pub fn contains(&self, other: &BBox) -> bool {
        self.min_x <= other.min_x
            && self.min_y <= other.min_y
            && self.max_x >= other.max_x
            && self.max_y >= other.max_y
    }

    /// Check if this bbox is contained by another
    pub fn within(&self, other: &BBox) -> bool {
        other.contains(self)
    }

    /// Check if this bbox is left of another
    pub fn left(&self, other: &BBox) -> bool {
        self.max_x < other.min_x
    }

    /// Check if this bbox is right of another
    pub fn right(&self, other: &BBox) -> bool {
        self.min_x > other.max_x
    }

    /// Check if this bbox is below another
    pub fn below(&self, other: &BBox) -> bool {
        self.max_y < other.min_y
    }

    /// Check if this bbox is above another
    pub fn above(&self, other: &BBox) -> bool {
        self.min_y > other.max_y
    }

    /// Calculate the area of the bounding box
    pub fn area(&self) -> f64 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }

    /// Calculate the union of two bounding boxes
    pub fn union(&self, other: &BBox) -> BBox {
        BBox::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    /// Calculate the enlargement needed to include another bbox
    pub fn enlargement(&self, other: &BBox) -> f64 {
        let union = self.union(other);
        union.area() - self.area()
    }
}

// ============================================================================
// BASIC GIST SUPPORT FUNCTIONS
// ============================================================================

/// Simple GiST compress function - converts geometry to bounding box
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_compress(geom: Geometry) -> BBox {
    BBox::from_geometry(&geom)
}

/// Simple compress function for PostgreSQL box type compatibility
/// Converts geometry to bounding box string in PostgreSQL box format
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_to_box_string(geom: Geometry) -> String {
    let (min_x, min_y, max_x, max_y) = geom.bounding_box();
    format!("(({},{}),({},{}))", min_x, min_y, max_x, max_y)
}

/// GiST union function - creates a bounding box that contains all input boxes
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_union(entries: Vec<BBox>) -> BBox {
    if entries.is_empty() {
        return BBox::new(0.0, 0.0, 0.0, 0.0);
    }

    let mut result = entries[0].clone();
    for entry in entries.iter().skip(1) {
        result = result.union(entry);
    }
    result
}

/// GiST penalty function - calculates the cost of inserting a new entry
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_penalty(original: BBox, new_entry: BBox) -> f32 {
    let enlargement = original.enlargement(&new_entry);
    enlargement as f32
}

/// GiST same function - determines if two entries are the same
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_same(a: BBox, b: BBox) -> bool {
    a == b
}

/// GiST decompress function - passthrough since we store bboxes directly
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_decompress(bbox: BBox) -> BBox {
    bbox
}

/// CRITICAL: GiST consistent function (Function 1) - This is REQUIRED by PostgreSQL
/// This function determines whether a query matches an index entry
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_consistent(
    key: BBox,
    query: BBox,
    strategy: i16,
    _subtype: pgrx::pg_sys::Oid,
    _recheck: bool,
) -> bool {
    // Strategy numbers correspond to different spatial operators
    // For PostgreSQL GiST, strategy 3 is typically && (overlaps)
    match strategy {
        3 => {
            // Strategy 3: && operator (bounding box overlap)
            key.overlaps(&query)
        }
        1 => {
            // Strategy 1: << operator (strictly left of)
            key.left(&query)
        }
        2 => {
            // Strategy 2: &< operator (does not extend to right of)
            key.max_x <= query.max_x
        }
        4 => {
            // Strategy 4: &> operator (does not extend to left of)
            key.min_x >= query.min_x
        }
        5 => {
            // Strategy 5: >> operator (strictly right of)
            key.right(&query)
        }
        6 => {
            // Strategy 6: ~= operator (same bounding box)
            key == query
        }
        7 => {
            // Strategy 7: ~ operator (contains)
            key.contains(&query)
        }
        8 => {
            // Strategy 8: @ operator (contained by)
            query.contains(&key)
        }
        10 => {
            // Strategy 10: <<| operator (strictly below)
            key.below(&query)
        }
        11 => {
            // Strategy 11: &<| operator (does not extend above)
            key.max_y <= query.max_y
        }
        12 => {
            // Strategy 12: |&> operator (does not extend below)
            key.min_y >= query.min_y
        }
        13 => {
            // Strategy 13: |>> operator (strictly above)
            key.above(&query)
        }
        _ => {
            // Default: assume overlap test for unknown strategies
            key.overlaps(&query)
        }
    }
}

/// GiST picksplit left function
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_picksplit_left(entries: Vec<BBox>) -> Vec<BBox> {
    if entries.len() <= 1 {
        return entries;
    }
    let mid = entries.len() / 2;
    entries[..mid].to_vec()
}

/// GiST picksplit right function
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_picksplit_right(entries: Vec<BBox>) -> Vec<BBox> {
    if entries.len() <= 1 {
        return Vec::new();
    }
    let mid = entries.len() / 2;
    entries[mid..].to_vec()
}

// ============================================================================
// SPATIAL INDEX FUNCTIONALITY (R*-TREE)
// ============================================================================

/// Wrapper for geometry with ID for use in spatial index
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryWithId {
    pub id: i64,
    pub geometry: Geometry,
    pub bbox: BBox,
}

impl GeometryWithId {
    pub fn new(id: i64, geometry: Geometry) -> Self {
        let bbox = BBox::from_geometry(&geometry);
        Self { id, geometry, bbox }
    }
}

/// Implement RTreeObject for our geometry wrapper to enable rstar indexing
impl RTreeObject for GeometryWithId {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.bbox.min_x, self.bbox.min_y],
            [self.bbox.max_x, self.bbox.max_y],
        )
    }
}

/// Implement PointDistance for distance-based queries
impl PointDistance for GeometryWithId {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let center_x = (self.bbox.min_x + self.bbox.max_x) / 2.0;
        let center_y = (self.bbox.min_y + self.bbox.max_y) / 2.0;

        let dx = center_x - point[0];
        let dy = center_y - point[1];

        dx * dx + dy * dy
    }
}

/// High-performance spatial index using R*-tree
pub struct SpatialIndex {
    rtree: RTree<GeometryWithId>,
}

impl SpatialIndex {
    /// Create a new empty spatial index
    pub fn new() -> Self {
        Self {
            rtree: RTree::new(),
        }
    }

    /// Create spatial index from a collection of geometries
    pub fn from_geometries(geometries: Vec<GeometryWithId>) -> Self {
        Self {
            rtree: RTree::bulk_load(geometries),
        }
    }

    /// Insert a geometry into the index
    pub fn insert(&mut self, geom_with_id: GeometryWithId) {
        self.rtree.insert(geom_with_id);
    }

    /// Remove a geometry from the index
    pub fn remove(&mut self, geom_with_id: &GeometryWithId) -> bool {
        self.rtree.remove(geom_with_id).is_some()
    }

    /// Find all geometries that intersect with the given bounding box
    pub fn query_bbox(&self, bbox: &BBox) -> Vec<&GeometryWithId> {
        let envelope = AABB::from_corners([bbox.min_x, bbox.min_y], [bbox.max_x, bbox.max_y]);
        self.rtree.locate_in_envelope(&envelope).collect()
    }

    /// Find the nearest neighbor to a point
    pub fn nearest_neighbor(&self, point: [f64; 2]) -> Option<&GeometryWithId> {
        self.rtree.nearest_neighbor(&point)
    }

    /// Find k nearest neighbors to a point
    pub fn k_nearest_neighbors(&self, point: [f64; 2], k: usize) -> Vec<&GeometryWithId> {
        self.rtree.nearest_neighbor_iter(&point).take(k).collect()
    }

    /// Find all geometries within distance of a point
    pub fn within_distance(&self, point: [f64; 2], distance: f64) -> Vec<&GeometryWithId> {
        self.rtree
            .locate_within_distance(point, distance * distance)
            .collect()
    }

    /// Get statistics about the index
    pub fn size(&self) -> usize {
        self.rtree.size()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.rtree.size() == 0
    }

    /// Get all geometries in the index
    pub fn iter(&self) -> impl Iterator<Item = &GeometryWithId> {
        self.rtree.iter()
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// POSTGRESQL FUNCTIONS FOR SPATIAL INDEXING DEMOS
// ============================================================================

/// PostgreSQL function to demonstrate R*-tree functionality
#[pg_extern(immutable, parallel_safe)]
pub fn create_rtree_demo(num_points: i32) -> String {
    let mut geometries = Vec::new();

    for i in 0..num_points {
        let x = (i as f64 * 1.123) % 100.0;
        let y = (i as f64 * 2.456) % 100.0;
        let geom = crate::functions::make_point(x, y);
        geometries.push(GeometryWithId::new(i as i64, geom));
    }

    let index = SpatialIndex::from_geometries(geometries);
    format!("Created R*-tree index with {} points", index.size())
}

/// PostgreSQL function to demonstrate nearest neighbor search
#[pg_extern(immutable, parallel_safe)]
pub fn rtree_nearest_neighbor_demo(num_points: i32, query_x: f64, query_y: f64) -> i64 {
    let mut geometries = Vec::new();

    for i in 0..num_points {
        let x = (i as f64 * 1.123) % 100.0;
        let y = (i as f64 * 2.456) % 100.0;
        let geom = crate::functions::make_point(x, y);
        geometries.push(GeometryWithId::new(i as i64, geom));
    }

    let index = SpatialIndex::from_geometries(geometries);

    if let Some(nearest) = index.nearest_neighbor([query_x, query_y]) {
        nearest.id
    } else {
        -1
    }
}

/// PostgreSQL function to demonstrate range query
#[pg_extern(immutable, parallel_safe)]
pub fn rtree_range_query_demo(
    num_points: i32,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> Vec<i64> {
    let mut geometries = Vec::new();

    for i in 0..num_points {
        let x = (i as f64 * 1.123) % 100.0;
        let y = (i as f64 * 2.456) % 100.0;
        let geom = crate::functions::make_point(x, y);
        geometries.push(GeometryWithId::new(i as i64, geom));
    }

    let index = SpatialIndex::from_geometries(geometries);
    let query_bbox = BBox::new(min_x, min_y, max_x, max_y);

    index
        .query_bbox(&query_bbox)
        .into_iter()
        .map(|g| g.id)
        .collect()
}

/// Input/Output functions for BBox
impl pgrx::InOutFuncs for BBox {
    fn input(input: &std::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        let input_str = input.to_str().expect("Invalid UTF-8 in bbox input");

        // Parse format: BOX(min_x min_y,max_x max_y)
        if let Some(coords_start) = input_str.find('(') {
            if let Some(coords_end) = input_str.find(')') {
                let coords_str = &input_str[coords_start + 1..coords_end];
                let parts: Vec<&str> = coords_str.split(',').collect();
                if parts.len() == 2 {
                    let min_coords: Vec<&str> = parts[0].trim().split_whitespace().collect();
                    let max_coords: Vec<&str> = parts[1].trim().split_whitespace().collect();

                    if min_coords.len() == 2 && max_coords.len() == 2 {
                        if let (Ok(min_x), Ok(min_y), Ok(max_x), Ok(max_y)) = (
                            min_coords[0].parse::<f64>(),
                            min_coords[1].parse::<f64>(),
                            max_coords[0].parse::<f64>(),
                            max_coords[1].parse::<f64>(),
                        ) {
                            return BBox::new(min_x, min_y, max_x, max_y);
                        }
                    }
                }
            }
        }

        // Fallback
        BBox::new(0.0, 0.0, 0.0, 0.0)
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        buffer.push_str(&format!(
            "BOX({} {},{} {})",
            self.min_x, self.min_y, self.max_x, self.max_y
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_creation() {
        let bbox = BBox::new(0.0, 0.0, 1.0, 1.0);
        assert_eq!(bbox.min_x, 0.0);
        assert_eq!(bbox.max_x, 1.0);
        assert_eq!(bbox.area(), 1.0);
    }

    #[test]
    fn test_bbox_overlaps() {
        let bbox1 = BBox::new(0.0, 0.0, 1.0, 1.0);
        let bbox2 = BBox::new(0.5, 0.5, 1.5, 1.5);
        let bbox3 = BBox::new(2.0, 2.0, 3.0, 3.0);

        assert!(bbox1.overlaps(&bbox2));
        assert!(!bbox1.overlaps(&bbox3));
    }

    #[test]
    fn test_spatial_index() {
        use crate::functions::make_point;

        let mut geometries = Vec::new();
        geometries.push(GeometryWithId::new(1, make_point(0.0, 0.0)));
        geometries.push(GeometryWithId::new(2, make_point(1.0, 1.0)));
        geometries.push(GeometryWithId::new(3, make_point(2.0, 2.0)));

        let index = SpatialIndex::from_geometries(geometries);
        assert_eq!(index.size(), 3);

        let nearest = index.nearest_neighbor([0.1, 0.1]).unwrap();
        assert_eq!(nearest.id, 1);
    }
}
