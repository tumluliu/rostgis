use crate::geometry::Geometry;
use pgrx::prelude::*;

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

use serde::{Deserialize, Serialize};

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
        return vec![];
    }

    let mid = entries.len() / 2;
    entries[mid..].to_vec()
}

/// GiST same function - determines if two entries are identical
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_same(a: BBox, b: BBox) -> bool {
    a == b
}

/// Extract bounding box from geometry for indexing
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_compress(geom: Geometry) -> BBox {
    BBox::from_geometry(&geom)
}

/// Decompress is usually the identity function for our case
#[pg_extern(immutable, parallel_safe)]
pub fn geometry_gist_decompress(bbox: BBox) -> BBox {
    bbox
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_creation() {
        let bbox = BBox::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(bbox.min_x, 0.0);
        assert_eq!(bbox.max_x, 10.0);
        assert_eq!(bbox.area(), 100.0);
    }

    #[test]
    fn test_bbox_overlaps() {
        let bbox1 = BBox::new(0.0, 0.0, 10.0, 10.0);
        let bbox2 = BBox::new(5.0, 5.0, 15.0, 15.0);
        let bbox3 = BBox::new(20.0, 20.0, 30.0, 30.0);

        assert!(bbox1.overlaps(&bbox2));
        assert!(!bbox1.overlaps(&bbox3));
    }

    #[test]
    fn test_bbox_contains() {
        let bbox1 = BBox::new(0.0, 0.0, 10.0, 10.0);
        let bbox2 = BBox::new(2.0, 2.0, 8.0, 8.0);
        let bbox3 = BBox::new(5.0, 5.0, 15.0, 15.0);

        assert!(bbox1.contains(&bbox2));
        assert!(!bbox1.contains(&bbox3));
    }

    #[test]
    fn test_bbox_union() {
        let bbox1 = BBox::new(0.0, 0.0, 5.0, 5.0);
        let bbox2 = BBox::new(3.0, 3.0, 8.0, 8.0);
        let union = bbox1.union(&bbox2);

        assert_eq!(union.min_x, 0.0);
        assert_eq!(union.min_y, 0.0);
        assert_eq!(union.max_x, 8.0);
        assert_eq!(union.max_y, 8.0);
    }
}
