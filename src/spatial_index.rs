use crate::geometry::Geometry;
use pgrx::prelude::*;
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

/// Bounding box type for spatial indexing
/// This represents a 2D rectangular bounding box with min/max x,y coordinates
#[derive(Debug, Clone, PartialEq, PostgresType, Serialize, Deserialize)]
#[inoutfuncs]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
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

    /// Calculate the intersection of two bounding boxes
    pub fn intersection(&self, other: &BBox) -> Option<BBox> {
        let min_x = self.min_x.max(other.min_x);
        let min_y = self.min_y.max(other.min_y);
        let max_x = self.max_x.min(other.max_x);
        let max_y = self.max_y.min(other.max_y);

        if min_x <= max_x && min_y <= max_y {
            Some(BBox::new(min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// Calculate the enlargement needed to include another bbox
    pub fn enlargement(&self, other: &BBox) -> f64 {
        let union = self.union(other);
        union.area() - self.area()
    }
}

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
        // Calculate distance from point to geometry's bounding box center
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
            .locate_within_distance(point, distance * distance) // rstar uses squared distance
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

/// PostgreSQL function to demonstrate R*-tree functionality
#[pg_extern(immutable, parallel_safe)]
pub fn create_rtree_demo(num_points: i32) -> String {
    let mut geometries = Vec::new();

    // Create test points
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

    // Create test points
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

    // Create test points
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

// GiST Index Support Functions
// These functions are needed for PostgreSQL's GiST index implementation

// Note: GiST support functions are complex to implement in pgrx due to
// PostgreSQL's internal types. The spatial operators below provide the
// foundation for spatial indexing.

/// GiST union function - creates a bounding box that contains all input boxes
/// This is used when building the index tree structure
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
/// This helps the index decide where to place new entries for optimal performance
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_penalty(original: BBox, new_entry: BBox) -> f32 {
    let enlargement = original.enlargement(&new_entry);
    enlargement as f32
}

/// GiST picksplit function - decides how to split an overfull index node
/// This is critical for index performance and balance
/// Note: Simplified implementation due to pgrx tuple return limitations
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_picksplit_left(entries: Vec<BBox>) -> Vec<BBox> {
    // Simple split implementation - returns left half
    if entries.len() < 2 {
        return entries;
    }

    let mid = entries.len() / 2;
    entries[0..mid].to_vec()
}

/// Returns the right half of a picksplit operation
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_picksplit_right(entries: Vec<BBox>) -> Vec<BBox> {
    // Simple split implementation - returns right half
    if entries.len() < 2 {
        return Vec::new();
    }

    let mid = entries.len() / 2;
    entries[mid..].to_vec()
}

/// GiST same function - determines if two entries are the same
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_same(a: BBox, b: BBox) -> bool {
    a == b
}

/// GiST compress function - converts geometry to indexable form
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_compress(geom: Geometry) -> BBox {
    BBox::from_geometry(&geom)
}

/// GiST decompress function - converts index form back to geometry (simplified)
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_decompress(bbox: BBox) -> BBox {
    bbox
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
    fn test_bbox_contains() {
        let bbox1 = BBox::new(0.0, 0.0, 2.0, 2.0);
        let bbox2 = BBox::new(0.5, 0.5, 1.5, 1.5);
        let bbox3 = BBox::new(1.0, 1.0, 3.0, 3.0);

        assert!(bbox1.contains(&bbox2));
        assert!(!bbox1.contains(&bbox3));
    }

    #[test]
    fn test_bbox_union() {
        let bbox1 = BBox::new(0.0, 0.0, 1.0, 1.0);
        let bbox2 = BBox::new(0.5, 0.5, 1.5, 1.5);
        let union = bbox1.union(&bbox2);

        assert_eq!(union.min_x, 0.0);
        assert_eq!(union.min_y, 0.0);
        assert_eq!(union.max_x, 1.5);
        assert_eq!(union.max_y, 1.5);
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

        // Test nearest neighbor
        let nearest = index.nearest_neighbor([0.1, 0.1]).unwrap();
        assert_eq!(nearest.id, 1);
    }
}
