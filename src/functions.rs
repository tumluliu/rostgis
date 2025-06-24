use crate::geometry::Geometry;
use geo::{Area, EuclideanDistance, EuclideanLength};
use geo_types::{LineString, Point, Polygon};

/// Create a Point geometry from WKT string
pub fn geometry_from_wkt(
    wkt_str: &str,
) -> Result<Geometry, Box<dyn std::error::Error + Send + Sync>> {
    // Simple WKT parser for basic geometries
    let wkt_str = wkt_str.trim().to_uppercase();

    if wkt_str.starts_with("POINT") {
        // Parse POINT(x y)
        if let Some(coords_start) = wkt_str.find('(') {
            if let Some(coords_end) = wkt_str.find(')') {
                let coords_str = &wkt_str[coords_start + 1..coords_end];
                let coords: Vec<&str> = coords_str.split_whitespace().collect();
                if coords.len() >= 2 {
                    let x: f64 = coords[0].parse().map_err(|_| "Invalid X coordinate")?;
                    let y: f64 = coords[1].parse().map_err(|_| "Invalid Y coordinate")?;
                    return Ok(Geometry::Point(Point::new(x, y), 0));
                }
            }
        }
        return Err("Invalid POINT WKT format".into());
    }

    if wkt_str.starts_with("LINESTRING") {
        // Parse LINESTRING(x1 y1, x2 y2, ...)
        if let Some(coords_start) = wkt_str.find('(') {
            if let Some(coords_end) = wkt_str.find(')') {
                let coords_str = &wkt_str[coords_start + 1..coords_end];
                let mut points = Vec::new();

                for point_str in coords_str.split(',') {
                    let coords: Vec<&str> = point_str.trim().split_whitespace().collect();
                    if coords.len() >= 2 {
                        let x: f64 = coords[0].parse().map_err(|_| "Invalid X coordinate")?;
                        let y: f64 = coords[1].parse().map_err(|_| "Invalid Y coordinate")?;
                        points.push((x, y));
                    }
                }

                if points.len() >= 2 {
                    let linestring = LineString::from(points);
                    return Ok(Geometry::LineString(linestring, 0));
                }
            }
        }
        return Err("Invalid LINESTRING WKT format".into());
    }

    if wkt_str.starts_with("POLYGON") {
        // Parse POLYGON((x1 y1, x2 y2, ...))
        if let Some(coords_start) = wkt_str.find("((") {
            if let Some(coords_end) = wkt_str.rfind("))") {
                let coords_str = &wkt_str[coords_start + 2..coords_end];
                let mut points = Vec::new();

                for point_str in coords_str.split(',') {
                    let coords: Vec<&str> = point_str.trim().split_whitespace().collect();
                    if coords.len() >= 2 {
                        let x: f64 = coords[0].parse().map_err(|_| "Invalid X coordinate")?;
                        let y: f64 = coords[1].parse().map_err(|_| "Invalid Y coordinate")?;
                        points.push((x, y));
                    }
                }

                if points.len() >= 4 {
                    let polygon = Polygon::new(LineString::from(points), vec![]);
                    return Ok(Geometry::Polygon(polygon, 0));
                }
            }
        }
        return Err("Invalid POLYGON WKT format".into());
    }

    Err("Unsupported geometry type".into())
}

/// Create a geometry from WKB hex string
pub fn geometry_from_wkb(
    _wkb_hex: &str,
) -> Result<Geometry, Box<dyn std::error::Error + Send + Sync>> {
    // For now, return an error as WKB parsing is complex
    // This would require implementing a full WKB parser
    Err("WKB parsing not yet implemented".into())
}

/// Create a Point geometry
pub fn make_point(x: f64, y: f64) -> Geometry {
    Geometry::Point(Point::new(x, y), 0)
}

/// Create a 3D Point geometry (Z coordinate stored as metadata for now)
pub fn make_point_z(x: f64, y: f64, _z: f64) -> Geometry {
    // For now, just create a 2D point
    // Full 3D support would require custom geometry types
    Geometry::Point(Point::new(x, y), 0)
}

/// Convert geometry to WKT string
pub fn geometry_as_text(geom: Geometry) -> String {
    geom.to_wkt()
}

/// Convert geometry to WKB hex string
pub fn geometry_as_wkb(geom: Geometry) -> String {
    // For now, return WKT as WKB is complex to implement
    // In a full implementation, this would convert to binary WKB format
    format!("WKB:{}", geom.to_wkt())
}

/// Convert geometry to GeoJSON string
pub fn geometry_as_geojson(geom: Geometry) -> String {
    match geom {
        Geometry::Point(point, _) => {
            format!(
                r#"{{"type":"Point","coordinates":[{},{}]}}"#,
                point.x(),
                point.y()
            )
        }
        Geometry::LineString(linestring, _) => {
            let coords: Vec<String> = linestring
                .coords()
                .map(|c| format!("[{},{}]", c.x, c.y))
                .collect();
            format!(
                r#"{{"type":"LineString","coordinates":[{}]}}"#,
                coords.join(",")
            )
        }
        Geometry::Polygon(polygon, _) => {
            let exterior: Vec<String> = polygon
                .exterior()
                .coords()
                .map(|c| format!("[{},{}]", c.x, c.y))
                .collect();
            format!(
                r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
                exterior.join(",")
            )
        }
        _ => format!(r#"{{"type":"Feature","geometry":null}}"#),
    }
}

/// Get X coordinate of a geometry (for Point types)
pub fn geometry_x(geom: Geometry) -> Option<f64> {
    geom.x()
}

/// Get Y coordinate of a geometry (for Point types)
pub fn geometry_y(geom: Geometry) -> Option<f64> {
    geom.y()
}

/// Get Z coordinate of a geometry (not implemented)
pub fn geometry_z(geom: Geometry) -> Option<f64> {
    geom.z()
}

/// Get geometry type as string
pub fn geometry_type(geom: Geometry) -> String {
    geom.geometry_type().to_string()
}

/// Get SRID of a geometry
pub fn geometry_srid(geom: Geometry) -> i32 {
    geom.srid()
}

/// Set SRID of a geometry
pub fn set_geometry_srid(geom: Geometry, srid: i32) -> Geometry {
    geom.with_srid(srid)
}

/// Check if two geometries are equal
pub fn geometries_equal(geom1: Geometry, geom2: Geometry) -> bool {
    geom1 == geom2
}

/// Calculate distance between two geometries
pub fn geometries_distance(geom1: Geometry, geom2: Geometry) -> f64 {
    match (geom1, geom2) {
        (Geometry::Point(p1, _), Geometry::Point(p2, _)) => p1.euclidean_distance(&p2),
        _ => 0.0, // Simplified for now
    }
}

/// Calculate area of a geometry
pub fn geometry_area(geom: Geometry) -> f64 {
    match geom {
        Geometry::Polygon(polygon, _) => polygon.unsigned_area(),
        Geometry::MultiPolygon(multipolygon, _) => multipolygon.unsigned_area(),
        _ => 0.0,
    }
}

/// Calculate length of a geometry
pub fn geometry_length(geom: Geometry) -> f64 {
    match geom {
        Geometry::LineString(linestring, _) => linestring.euclidean_length(),
        Geometry::MultiLineString(multilinestring, _) => multilinestring.euclidean_length(),
        Geometry::Polygon(polygon, _) => polygon.exterior().euclidean_length(),
        _ => 0.0,
    }
}

/// Calculate perimeter of a geometry
pub fn geometry_perimeter(geom: Geometry) -> f64 {
    match geom {
        Geometry::Polygon(polygon, _) => {
            let mut perimeter = polygon.exterior().euclidean_length();
            for interior in polygon.interiors() {
                perimeter += interior.euclidean_length();
            }
            perimeter
        }
        Geometry::MultiPolygon(multipolygon, _) => multipolygon
            .iter()
            .map(|p| geometry_perimeter(Geometry::Polygon(p.clone(), 0)))
            .sum(),
        _ => geometry_length(geom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_point() {
        let point = make_point(1.0, 2.0);
        assert_eq!(geometry_x(point.clone()).unwrap(), 1.0);
        assert_eq!(geometry_y(point.clone()).unwrap(), 2.0);
        assert_eq!(geometry_type(point), "ST_Point");
    }

    #[test]
    fn test_geometry_from_wkt() {
        let result = geometry_from_wkt("POINT(1 2)");
        assert!(result.is_ok());
        let geom = result.unwrap();
        assert_eq!(geometry_x(geom.clone()).unwrap(), 1.0);
        assert_eq!(geometry_y(geom.clone()).unwrap(), 2.0);
    }

    #[test]
    fn test_geometry_as_text() {
        let point = make_point(1.0, 2.0);
        let wkt = geometry_as_text(point);
        assert_eq!(wkt, "POINT(1 2)");
    }

    #[test]
    fn test_geometries_distance() {
        let point1 = make_point(0.0, 0.0);
        let point2 = make_point(3.0, 4.0);
        let distance = geometries_distance(point1, point2);
        assert!((distance - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometry_as_geojson() {
        let point = make_point(1.0, 2.0);
        let geojson = geometry_as_geojson(point);
        assert_eq!(geojson, r#"{"type":"Point","coordinates":[1,2]}"#);
    }

    #[test]
    fn test_srid_operations() {
        let point = make_point(1.0, 2.0);
        assert_eq!(geometry_srid(point.clone()), 0);

        let point_with_srid = set_geometry_srid(point, 4326);
        assert_eq!(geometry_srid(point_with_srid), 4326);
    }
}
