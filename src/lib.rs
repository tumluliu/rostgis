use pgrx::prelude::*;
// serde imports removed as they're not needed in main lib

::pgrx::pg_module_magic!();

// Re-export modules
pub mod functions;
pub mod geometry;
pub mod spatial_index;
pub mod utils;
pub mod vectorized_ops;

use functions::*;
use geometry::Geometry;
// Import spatial indexing support
// Note: GistBBox functions available but using simpler bbox approach for now
use spatial_index::BBox;

// Extension initialization
#[pg_extern]
fn rostgis_version() -> &'static str {
    "RostGIS 0.1.0 - PostGIS-compatible spatial extension for PostgreSQL"
}

// Core geometry creation functions
#[pg_extern]
fn st_geomfromtext(wkt: &str) -> Result<Geometry, Box<dyn std::error::Error + Send + Sync>> {
    geometry_from_wkt(wkt)
}

#[pg_extern]
fn st_geomfromwkt(wkt: &str) -> Result<Geometry, Box<dyn std::error::Error + Send + Sync>> {
    geometry_from_wkt(wkt)
}

#[pg_extern]
fn st_geomfromwkb(wkb_hex: &str) -> Result<Geometry, Box<dyn std::error::Error + Send + Sync>> {
    geometry_from_wkb(wkb_hex)
}

#[pg_extern]
fn st_makepoint(x: f64, y: f64) -> Geometry {
    make_point(x, y)
}

#[pg_extern]
fn st_point(x: f64, y: f64) -> Geometry {
    make_point(x, y)
}

#[pg_extern]
fn st_makepointz(x: f64, y: f64, z: f64) -> Geometry {
    make_point_z(x, y, z)
}

// Geometry output functions
#[pg_extern]
fn st_astext(geom: Geometry) -> String {
    geometry_as_text(geom)
}

#[pg_extern]
fn st_aswkt(geom: Geometry) -> String {
    geometry_as_text(geom)
}

#[pg_extern]
fn st_aswkb(geom: Geometry) -> String {
    geometry_as_wkb(geom)
}

#[pg_extern]
fn st_asgeojson(geom: Geometry) -> String {
    geometry_as_geojson(geom)
}

// Geometry property functions
#[pg_extern]
fn st_x(geom: Geometry) -> Option<f64> {
    geometry_x(geom)
}

#[pg_extern]
fn st_y(geom: Geometry) -> Option<f64> {
    geometry_y(geom)
}

#[pg_extern]
fn st_z(geom: Geometry) -> Option<f64> {
    geometry_z(geom)
}

#[pg_extern]
fn st_geometrytype(geom: Geometry) -> String {
    geometry_type(geom)
}

#[pg_extern]
fn st_srid(geom: Geometry) -> i32 {
    geometry_srid(geom)
}

#[pg_extern]
fn st_setsrid(geom: Geometry, srid: i32) -> Geometry {
    set_geometry_srid(geom, srid)
}

// Geometry relationship functions
#[pg_extern]
fn st_equals(geom1: Geometry, geom2: Geometry) -> bool {
    geometries_equal(geom1, geom2)
}

#[pg_extern]
fn st_distance(geom1: Geometry, geom2: Geometry) -> f64 {
    geometries_distance(geom1, geom2)
}

#[pg_extern]
fn st_area(geom: Geometry) -> f64 {
    geometry_area(geom)
}

#[pg_extern]
fn st_length(geom: Geometry) -> f64 {
    geometry_length(geom)
}

#[pg_extern]
fn st_perimeter(geom: Geometry) -> f64 {
    geometry_perimeter(geom)
}

// Spatial indexing functions
#[pg_extern]
fn st_envelope(geom: Geometry) -> BBox {
    BBox::from_geometry(&geom)
}

/// Simple compress function for spatial indexing
/// Converts geometry to bounding box string in PostgreSQL box format
#[pg_extern(immutable, parallel_safe)]
fn geometry_to_box(geom: Geometry) -> String {
    let (min_x, min_y, max_x, max_y) = geom.bounding_box();
    format!("(({},{}),({},{}))", min_x, min_y, max_x, max_y)
}

// Spatial operators for indexing support
// These operators work on bounding boxes and can utilize spatial indexes

/// Bounding box overlap operator (&&)
/// This is the most commonly used spatial operator for index acceleration
#[pg_operator(immutable, parallel_safe)]
#[opname(&&)]
fn geometry_overlap(left: Geometry, right: Geometry) -> bool {
    left.bbox_overlaps(&right)
}

/// Bounding box left operator (<<)
#[pg_operator(immutable, parallel_safe)]
#[opname(<<)]
fn geometry_left(left: Geometry, right: Geometry) -> bool {
    left.bbox_left(&right)
}

/// Bounding box right operator (>>)
#[pg_operator(immutable, parallel_safe)]
#[opname(>>)]
fn geometry_right(left: Geometry, right: Geometry) -> bool {
    left.bbox_right(&right)
}

/// Bounding box below operator (<<|)
#[pg_operator(immutable, parallel_safe)]
#[opname(<<|)]
fn geometry_below(left: Geometry, right: Geometry) -> bool {
    left.bbox_below(&right)
}

/// Bounding box above operator (|>>)
#[pg_operator(immutable, parallel_safe)]
#[opname(|>>)]
fn geometry_above(left: Geometry, right: Geometry) -> bool {
    left.bbox_above(&right)
}

/// Bounding box contains operator (~)
#[pg_operator(immutable, parallel_safe)]
#[opname(~)]
fn geometry_contains_bbox(left: Geometry, right: Geometry) -> bool {
    left.bbox_contains(&right)
}

/// Bounding box contained by operator (@)
#[pg_operator(immutable, parallel_safe)]
#[opname(@)]
fn geometry_within_bbox(left: Geometry, right: Geometry) -> bool {
    left.bbox_within(&right)
}

/// Overlap left operator (&<)
#[pg_operator(immutable, parallel_safe)]
#[opname(&<)]
fn geometry_overleft(left: Geometry, right: Geometry) -> bool {
    !left.bbox_right(&right)
}

/// Overlap right operator (&>)
#[pg_operator(immutable, parallel_safe)]
#[opname(&>)]
fn geometry_overright(left: Geometry, right: Geometry) -> bool {
    !left.bbox_left(&right)
}

/// Overlap below operator (&<|)
#[pg_operator(immutable, parallel_safe)]
#[opname(&<|)]
fn geometry_overbelow(left: Geometry, right: Geometry) -> bool {
    !left.bbox_above(&right)
}

/// Overlap above operator (|&>)
#[pg_operator(immutable, parallel_safe)]
#[opname(|&>)]
fn geometry_overabove(left: Geometry, right: Geometry) -> bool {
    !left.bbox_below(&right)
}

/// Same bounding box operator (~=)
#[pg_operator(immutable, parallel_safe)]
#[opname(~=)]
fn geometry_same_bbox(left: Geometry, right: Geometry) -> bool {
    let (min_x1, min_y1, max_x1, max_y1) = left.bounding_box();
    let (min_x2, min_y2, max_x2, max_y2) = right.bounding_box();

    (min_x1 - min_x2).abs() < f64::EPSILON
        && (min_y1 - min_y2).abs() < f64::EPSILON
        && (max_x1 - max_x2).abs() < f64::EPSILON
        && (max_y1 - max_y2).abs() < f64::EPSILON
}

// Spatial relationship functions that can use indexes
#[pg_extern]
fn st_intersects(geom1: Geometry, geom2: Geometry) -> bool {
    // First check bounding box overlap (can use index)
    if !geom1.bbox_overlaps(&geom2) {
        return false;
    }

    // For now, if bboxes overlap, assume intersection
    // In a full implementation, this would do exact geometric intersection testing
    true
}

#[pg_extern]
fn st_contains(geom1: Geometry, geom2: Geometry) -> bool {
    // First check bounding box containment (can use index)
    if !geom1.bbox_contains(&geom2) {
        return false;
    }

    // For now, if bbox contains, assume geometric containment
    // In a full implementation, this would do exact geometric containment testing
    true
}

#[pg_extern]
fn st_within(geom1: Geometry, geom2: Geometry) -> bool {
    st_contains(geom2, geom1)
}

#[pg_extern]
fn st_dwithin(geom1: Geometry, geom2: Geometry, distance: f64) -> bool {
    // This is a simplified implementation
    // A proper implementation would expand the bounding box by the distance
    let actual_distance = geometries_distance(geom1, geom2);
    actual_distance <= distance
}

// Test module
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::*;

    #[pg_test]
    fn test_rostgis_version() {
        let version = crate::rostgis_version();
        assert!(version.contains("RostGIS"));
        assert!(version.contains("0.1.0"));
    }

    #[pg_test]
    fn test_st_makepoint() {
        let point = crate::st_makepoint(1.0, 2.0);
        assert_eq!(crate::st_x(point.clone()).unwrap(), 1.0);
        assert_eq!(crate::st_y(point.clone()).unwrap(), 2.0);
        assert_eq!(crate::st_geometrytype(point), "ST_Point");
    }

    #[pg_test]
    fn test_st_geomfromtext() {
        let result = crate::st_geomfromtext("POINT(1 2)");
        assert!(result.is_ok());
        let geom = result.unwrap();
        assert_eq!(crate::st_x(geom.clone()).unwrap(), 1.0);
        assert_eq!(crate::st_y(geom.clone()).unwrap(), 2.0);
    }

    #[pg_test]
    fn test_st_astext() {
        let point = crate::st_makepoint(1.0, 2.0);
        let wkt = crate::st_astext(point);
        assert_eq!(wkt, "POINT(1 2)");
    }

    #[pg_test]
    fn test_st_distance() {
        let point1 = crate::st_makepoint(0.0, 0.0);
        let point2 = crate::st_makepoint(3.0, 4.0);
        let distance = crate::st_distance(point1, point2);
        assert!((distance - 5.0).abs() < 1e-10);
    }

    #[pg_test]
    fn test_st_srid() {
        let point = crate::st_makepoint(1.0, 2.0);
        assert_eq!(crate::st_srid(point), 0); // Default SRID

        let point_with_srid = crate::st_setsrid(crate::st_makepoint(1.0, 2.0), 4326);
        assert_eq!(crate::st_srid(point_with_srid), 4326);
    }

    #[pg_test]
    fn test_spatial_operators() {
        let point1 = crate::st_makepoint(0.0, 0.0);
        let point2 = crate::st_makepoint(1.0, 1.0);
        let point3 = crate::st_makepoint(10.0, 10.0);

        // Test overlap (points always overlap themselves for bounding box purposes)
        assert!(crate::geometry_overlap(point1.clone(), point1.clone()));

        // Test spatial relationships
        assert!(crate::st_intersects(point1.clone(), point1.clone()));
        assert!(crate::st_dwithin(point1.clone(), point2.clone(), 2.0));
        assert!(!crate::st_dwithin(point1.clone(), point3.clone(), 1.0));
    }

    #[pg_test]
    fn test_st_envelope() {
        let point = crate::st_makepoint(1.0, 2.0);
        let bbox = crate::st_envelope(point);
        // For a point, the envelope should be the point coordinates
        assert_eq!(bbox.min_x, 1.0);
        assert_eq!(bbox.min_y, 2.0);
        assert_eq!(bbox.max_x, 1.0);
        assert_eq!(bbox.max_y, 2.0);
    }
}

/// This module is required by `cargo pgrx test` invocations.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
