use geo_types::{LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use pgrx::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PostGIS-compatible Geometry type
/// This enum represents all supported geometry types
#[derive(Debug, Clone, PartialEq, PostgresType, Serialize, Deserialize)]
#[inoutfuncs]
pub enum Geometry {
    Point(Point<f64>, i32), // (point, srid)
    LineString(LineString<f64>, i32),
    Polygon(Polygon<f64>, i32),
    MultiPoint(MultiPoint<f64>, i32),
    MultiLineString(MultiLineString<f64>, i32),
    MultiPolygon(MultiPolygon<f64>, i32),
    GeometryCollection(Vec<Geometry>, i32),
}

impl Geometry {
    /// Get the SRID of the geometry
    pub fn srid(&self) -> i32 {
        match self {
            Geometry::Point(_, srid) => *srid,
            Geometry::LineString(_, srid) => *srid,
            Geometry::Polygon(_, srid) => *srid,
            Geometry::MultiPoint(_, srid) => *srid,
            Geometry::MultiLineString(_, srid) => *srid,
            Geometry::MultiPolygon(_, srid) => *srid,
            Geometry::GeometryCollection(_, srid) => *srid,
        }
    }

    /// Set the SRID of the geometry
    pub fn with_srid(mut self, srid: i32) -> Self {
        match &mut self {
            Geometry::Point(_, s) => *s = srid,
            Geometry::LineString(_, s) => *s = srid,
            Geometry::Polygon(_, s) => *s = srid,
            Geometry::MultiPoint(_, s) => *s = srid,
            Geometry::MultiLineString(_, s) => *s = srid,
            Geometry::MultiPolygon(_, s) => *s = srid,
            Geometry::GeometryCollection(_, s) => *s = srid,
        }
        self
    }

    /// Get the geometry type as a string (PostGIS compatible)
    pub fn geometry_type(&self) -> &'static str {
        match self {
            Geometry::Point(_, _) => "ST_Point",
            Geometry::LineString(_, _) => "ST_LineString",
            Geometry::Polygon(_, _) => "ST_Polygon",
            Geometry::MultiPoint(_, _) => "ST_MultiPoint",
            Geometry::MultiLineString(_, _) => "ST_MultiLineString",
            Geometry::MultiPolygon(_, _) => "ST_MultiPolygon",
            Geometry::GeometryCollection(_, _) => "ST_GeometryCollection",
        }
    }

    /// Check if geometry is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Geometry::Point(_, _) => false, // Points are never empty in this implementation
            Geometry::LineString(ls, _) => ls.0.is_empty(),
            Geometry::Polygon(p, _) => p.exterior().0.is_empty(),
            Geometry::MultiPoint(mp, _) => mp.0.is_empty(),
            Geometry::MultiLineString(mls, _) => mls.0.is_empty(),
            Geometry::MultiPolygon(mp, _) => mp.0.is_empty(),
            Geometry::GeometryCollection(gc, _) => gc.is_empty(),
        }
    }

    /// Get X coordinate (for Point geometries)
    pub fn x(&self) -> Option<f64> {
        match self {
            Geometry::Point(point, _) => Some(point.x()),
            _ => None,
        }
    }

    /// Get Y coordinate (for Point geometries)
    pub fn y(&self) -> Option<f64> {
        match self {
            Geometry::Point(point, _) => Some(point.y()),
            _ => None,
        }
    }

    /// Get Z coordinate (not implemented yet, returns None)
    pub fn z(&self) -> Option<f64> {
        // Z coordinate support would require extending geo-types or using a different approach
        None
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_wkt())
    }
}

impl Geometry {
    /// Convert geometry to WKT string
    pub fn to_wkt(&self) -> String {
        match self {
            Geometry::Point(point, _) => {
                format!("POINT({} {})", point.x(), point.y())
            }
            Geometry::LineString(linestring, _) => {
                let coords: Vec<String> = linestring
                    .coords()
                    .map(|c| format!("{} {}", c.x, c.y))
                    .collect();
                format!("LINESTRING({})", coords.join(","))
            }
            Geometry::Polygon(polygon, _) => {
                let exterior: Vec<String> = polygon
                    .exterior()
                    .coords()
                    .map(|c| format!("{} {}", c.x, c.y))
                    .collect();
                let mut wkt = format!("POLYGON(({})", exterior.join(","));

                for interior in polygon.interiors() {
                    let interior_coords: Vec<String> = interior
                        .coords()
                        .map(|c| format!("{} {}", c.x, c.y))
                        .collect();
                    wkt.push_str(&format!(",({})", interior_coords.join(",")));
                }
                wkt.push(')');
                wkt
            }
            Geometry::MultiPoint(multipoint, _) => {
                let points: Vec<String> = multipoint
                    .iter()
                    .map(|p| format!("({} {})", p.x(), p.y()))
                    .collect();
                format!("MULTIPOINT({})", points.join(","))
            }
            Geometry::MultiLineString(multilinestring, _) => {
                let linestrings: Vec<String> = multilinestring
                    .iter()
                    .map(|ls| {
                        let coords: Vec<String> =
                            ls.coords().map(|c| format!("{} {}", c.x, c.y)).collect();
                        format!("({})", coords.join(","))
                    })
                    .collect();
                format!("MULTILINESTRING({})", linestrings.join(","))
            }
            Geometry::MultiPolygon(multipolygon, _) => {
                let polygons: Vec<String> = multipolygon
                    .iter()
                    .map(|poly| {
                        let exterior: Vec<String> = poly
                            .exterior()
                            .coords()
                            .map(|c| format!("{} {}", c.x, c.y))
                            .collect();
                        let mut poly_wkt = format!("(({})", exterior.join(","));

                        for interior in poly.interiors() {
                            let interior_coords: Vec<String> = interior
                                .coords()
                                .map(|c| format!("{} {}", c.x, c.y))
                                .collect();
                            poly_wkt.push_str(&format!(",({})", interior_coords.join(",")));
                        }
                        poly_wkt.push(')');
                        poly_wkt
                    })
                    .collect();
                format!("MULTIPOLYGON({})", polygons.join(","))
            }
            Geometry::GeometryCollection(geometries, _) => {
                let geoms: Vec<String> = geometries.iter().map(|g| g.to_wkt()).collect();
                format!("GEOMETRYCOLLECTION({})", geoms.join(","))
            }
        }
    }
}

/// Input/Output functions for PostgreSQL integration
impl pgrx::InOutFuncs for Geometry {
    fn input(input: &std::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        let input_str = input.to_str().expect("Invalid UTF-8 in geometry input");

        // Simple WKT parsing for input
        if input_str.trim().to_uppercase().starts_with("POINT") {
            // Parse POINT(x y)
            if let Some(coords_start) = input_str.find('(') {
                if let Some(coords_end) = input_str.find(')') {
                    let coords_str = &input_str[coords_start + 1..coords_end];
                    let coords: Vec<&str> = coords_str.split_whitespace().collect();
                    if coords.len() >= 2 {
                        if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>())
                        {
                            return Geometry::Point(Point::new(x, y), 0);
                        }
                    }
                }
            }
        }

        // Fallback: create a point at origin
        Geometry::Point(Point::new(0.0, 0.0), 0)
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        buffer.push_str(&self.to_wkt());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_type() {
        let point = Geometry::Point(Point::new(1.0, 2.0), 0);
        assert_eq!(point.geometry_type(), "ST_Point");
    }

    #[test]
    fn test_point_coordinates() {
        let point = Geometry::Point(Point::new(1.0, 2.0), 0);
        assert_eq!(point.x(), Some(1.0));
        assert_eq!(point.y(), Some(2.0));
    }

    #[test]
    fn test_srid_operations() {
        let point = Geometry::Point(Point::new(1.0, 2.0), 0);
        assert_eq!(point.srid(), 0);

        let point_with_srid = point.with_srid(4326);
        assert_eq!(point_with_srid.srid(), 4326);
    }

    #[test]
    fn test_wkt_output() {
        let point = Geometry::Point(Point::new(1.0, 2.0), 0);
        assert_eq!(point.to_wkt(), "POINT(1 2)");
    }
}
