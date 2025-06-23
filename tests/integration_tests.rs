use rostgis::functions::*;
use rostgis::geometry::Geometry;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_point_workflow() {
        // Create a point
        let point = make_point(1.0, 2.0);

        // Test coordinates
        assert_eq!(geometry_x(point.clone()).unwrap(), 1.0);
        assert_eq!(geometry_y(point.clone()).unwrap(), 2.0);
        assert_eq!(geometry_z(point.clone()), None);

        // Test type
        assert_eq!(geometry_type(point.clone()), "ST_Point");

        // Test SRID
        assert_eq!(geometry_srid(point.clone()), 0);

        // Test WKT output
        assert_eq!(geometry_as_text(point.clone()), "POINT(1 2)");

        // Test GeoJSON output
        assert_eq!(
            geometry_as_geojson(point),
            r#"{"type":"Point","coordinates":[1,2]}"#
        );
    }

    #[test]
    fn test_wkt_parsing_workflow() {
        // Test point parsing
        let point_result = geometry_from_wkt("POINT(1 2)");
        assert!(point_result.is_ok());
        let point = point_result.unwrap();
        assert_eq!(geometry_x(point.clone()).unwrap(), 1.0);
        assert_eq!(geometry_y(point.clone()).unwrap(), 2.0);

        // Test linestring parsing
        let linestring_result = geometry_from_wkt("LINESTRING(0 0, 1 1, 2 2)");
        assert!(linestring_result.is_ok());
        let linestring = linestring_result.unwrap();
        assert_eq!(geometry_type(linestring.clone()), "ST_LineString");
        assert!(geometry_length(linestring) > 0.0);

        // Test polygon parsing
        let polygon_result = geometry_from_wkt("POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))");
        assert!(polygon_result.is_ok());
        let polygon = polygon_result.unwrap();
        assert_eq!(geometry_type(polygon.clone()), "ST_Polygon");
        assert_eq!(geometry_area(polygon.clone()), 1.0);
        assert_eq!(geometry_perimeter(polygon), 4.0);
    }

    #[test]
    fn test_distance_calculations() {
        // Test simple distance
        let point1 = make_point(0.0, 0.0);
        let point2 = make_point(3.0, 4.0);
        let distance = geometries_distance(point1, point2);
        assert!((distance - 5.0).abs() < 1e-10);

        // Test same point distance
        let point_a = make_point(1.0, 1.0);
        let point_b = make_point(1.0, 1.0);
        let zero_distance = geometries_distance(point_a, point_b);
        assert!(zero_distance.abs() < 1e-10);
    }

    #[test]
    fn test_srid_operations() {
        let point = make_point(1.0, 2.0);

        // Test default SRID
        assert_eq!(geometry_srid(point.clone()), 0);

        // Test setting SRID
        let point_wgs84 = set_geometry_srid(point.clone(), 4326);
        assert_eq!(geometry_srid(point_wgs84), 4326);

        // Test setting multiple SRIDs
        let point_utm = set_geometry_srid(point, 32633);
        assert_eq!(geometry_srid(point_utm), 32633);
    }

    #[test]
    fn test_geometry_equality() {
        let point1 = make_point(1.0, 2.0);
        let point2 = make_point(1.0, 2.0);
        let point3 = make_point(2.0, 3.0);

        // Test equal geometries
        assert!(geometries_equal(point1.clone(), point2));

        // Test different geometries
        assert!(!geometries_equal(point1, point3));
    }

    #[test]
    fn test_polygon_measurements() {
        // Test square polygon
        let square = geometry_from_wkt("POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))").unwrap();
        assert_eq!(geometry_area(square.clone()), 100.0);
        assert_eq!(geometry_perimeter(square), 40.0);

        // Test triangle polygon
        let triangle = geometry_from_wkt("POLYGON((0 0, 3 0, 0 4, 0 0))").unwrap();
        assert_eq!(geometry_area(triangle.clone()), 6.0);
        assert!((geometry_perimeter(triangle) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_linestring_measurements() {
        // Test simple linestring
        let line = geometry_from_wkt("LINESTRING(0 0, 3 4)").unwrap();
        assert!((geometry_length(line) - 5.0).abs() < 1e-10);

        // Test multi-segment linestring
        let multi_line = geometry_from_wkt("LINESTRING(0 0, 1 0, 1 1, 0 1)").unwrap();
        assert_eq!(geometry_length(multi_line), 3.0);
    }

    #[test]
    fn test_invalid_wkt() {
        // Test invalid WKT
        let result = geometry_from_wkt("INVALID(1 2)");
        assert!(result.is_err());

        // Test empty string
        let result = geometry_from_wkt("");
        assert!(result.is_err());

        // Test malformed WKT
        let result = geometry_from_wkt("POINT(1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_output_format_consistency() {
        let point = make_point(1.0, 2.0);
        let wkt = geometry_as_text(point.clone());

        // Parse the output back
        let reparsed = geometry_from_wkt(&wkt).unwrap();

        // Should be equal
        assert!(geometries_equal(point, reparsed));
    }

    #[test]
    fn test_3d_point_basic() {
        let point_3d = make_point_z(1.0, 2.0, 3.0);

        // Should still have X and Y coordinates
        assert_eq!(geometry_x(point_3d.clone()).unwrap(), 1.0);
        assert_eq!(geometry_y(point_3d.clone()).unwrap(), 2.0);

        // Z coordinate not fully implemented yet
        assert_eq!(geometry_z(point_3d.clone()), None);

        // Should still be a point type
        assert_eq!(geometry_type(point_3d), "ST_Point");
    }

    #[test]
    fn test_multigeometry_types() {
        // Test multipoint
        let multipoint_result = geometry_from_wkt("MULTIPOINT((0 0), (1 1))");
        assert!(multipoint_result.is_ok());
        let multipoint = multipoint_result.unwrap();
        assert_eq!(geometry_type(multipoint), "ST_MultiPoint");

        // Test multilinestring
        let multilinestring_result = geometry_from_wkt("MULTILINESTRING((0 0, 1 1), (2 2, 3 3))");
        assert!(multilinestring_result.is_ok());
        let multilinestring = multilinestring_result.unwrap();
        assert_eq!(geometry_type(multilinestring), "ST_MultiLineString");

        // Test multipolygon
        let multipolygon_result = geometry_from_wkt(
            "MULTIPOLYGON(((0 0, 1 0, 1 1, 0 1, 0 0)), ((2 2, 3 2, 3 3, 2 3, 2 2)))",
        );
        assert!(multipolygon_result.is_ok());
        let multipolygon = multipolygon_result.unwrap();
        assert_eq!(geometry_type(multipolygon), "ST_MultiPolygon");
    }
}
