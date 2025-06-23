-- Test basic RostGIS functionality
-- This file tests PostGIS compatibility

-- Create extension
CREATE EXTENSION IF NOT EXISTS rostgis;

-- Test version function
SELECT rostgis_version();

-- Test point creation
SELECT ST_MakePoint(1.0, 2.0) AS point;
SELECT ST_Point(1.0, 2.0) AS point_alias;

-- Test WKT parsing
SELECT ST_GeomFromText('POINT(1 2)') AS point_from_wkt;
SELECT ST_GeomFromWKT('POINT(1 2)') AS point_from_wkt_alias;

-- Test coordinate extraction
SELECT ST_X(ST_MakePoint(1.0, 2.0)) AS x_coord;
SELECT ST_Y(ST_MakePoint(1.0, 2.0)) AS y_coord;
SELECT ST_Z(ST_MakePoint(1.0, 2.0)) AS z_coord; -- Should be NULL

-- Test geometry type
SELECT ST_GeometryType(ST_MakePoint(1.0, 2.0)) AS geom_type;

-- Test SRID operations
SELECT ST_SRID(ST_MakePoint(1.0, 2.0)) AS default_srid;
SELECT ST_SRID(ST_SetSRID(ST_MakePoint(1.0, 2.0), 4326)) AS set_srid;

-- Test output formats
SELECT ST_AsText(ST_MakePoint(1.0, 2.0)) AS wkt_output;
SELECT ST_AsWKT(ST_MakePoint(1.0, 2.0)) AS wkt_output_alias;
SELECT ST_AsWKB(ST_MakePoint(1.0, 2.0)) AS wkb_output;
SELECT ST_AsGeoJSON(ST_MakePoint(1.0, 2.0)) AS geojson_output;

-- Test distance calculation
SELECT ST_Distance(
    ST_MakePoint(0, 0),
    ST_MakePoint(3, 4)
) AS euclidean_distance;

-- Test geometry equality
SELECT ST_Equals(
    ST_MakePoint(1, 2),
    ST_MakePoint(1, 2)
) AS geometries_equal;

SELECT ST_Equals(
    ST_MakePoint(1, 2),
    ST_MakePoint(2, 1)
) AS geometries_not_equal;

-- Test with more complex geometries
SELECT ST_GeomFromText('LINESTRING(0 0, 1 1, 2 2)') AS linestring;
SELECT ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))') AS polygon;

-- Test area and length calculations
SELECT ST_Area(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))')) AS polygon_area;
SELECT ST_Length(ST_GeomFromText('LINESTRING(0 0, 3 4)')) AS linestring_length;
SELECT ST_Perimeter(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))')) AS polygon_perimeter;

-- Test 3D point (basic support)
SELECT ST_MakePointZ(1.0, 2.0, 3.0) AS point_3d;
SELECT ST_AsText(ST_MakePointZ(1.0, 2.0, 3.0)) AS point_3d_wkt; 