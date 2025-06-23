use pgrx::prelude::*;
// serde imports removed as they're not needed in main lib

::pgrx::pg_module_magic!();

// Re-export modules
pub mod functions;
pub mod geometry;
pub mod utils;

use functions::*;
use geometry::Geometry;

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

// Test module
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::*;
    use pgrx::prelude::*;

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
