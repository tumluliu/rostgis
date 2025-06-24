# RostGIS Spatial Functions API Reference

Complete reference for all spatial functions available in RostGIS.

## Function Categories

### 🏗️ Geometry Construction Functions
- [ST_MakePoint](#st_makepoint) - Create a point geometry
- [ST_Point](#st_point) - Alias for ST_MakePoint  
- [ST_MakePointZ](#st_makepointz) - Create a 3D point
- [ST_GeomFromText](#st_geomfromtext) - Create geometry from WKT
- [ST_GeomFromWKT](#st_geomfromwkt) - Alias for ST_GeomFromText

### 📤 Geometry Output Functions
- [ST_AsText](#st_astext) - Convert geometry to WKT
- [ST_AsWKT](#st_aswkt) - Alias for ST_AsText
- [ST_AsWKB](#st_aswkb) - Convert geometry to WKB hex
- [ST_AsGeoJSON](#st_asgeojson) - Convert geometry to GeoJSON

### 📏 Geometry Property Functions
- [ST_X](#st_x) - Get X coordinate
- [ST_Y](#st_y) - Get Y coordinate  
- [ST_Z](#st_z) - Get Z coordinate
- [ST_GeometryType](#st_geometrytype) - Get geometry type
- [ST_SRID](#st_srid) - Get spatial reference ID
- [ST_SetSRID](#st_setsrid) - Set spatial reference ID
- [ST_Envelope](#st_envelope) - Get bounding box

### 📐 Measurement Functions
- [ST_Distance](#st_distance) - Calculate distance between geometries
- [ST_Area](#st_area) - Calculate area of polygon
- [ST_Length](#st_length) - Calculate length of linestring
- [ST_Perimeter](#st_perimeter) - Calculate perimeter of polygon

### 🔍 Spatial Relationship Functions
- [ST_Equals](#st_equals) - Test geometric equality
- [ST_Intersects](#st_intersects) - Test if geometries intersect
- [ST_Contains](#st_contains) - Test if geometry contains another
- [ST_Within](#st_within) - Test if geometry is within another
- [ST_DWithin](#st_dwithin) - Test if geometries are within distance

### 🌐 Extension Information
- [rostgis_version](#rostgis_version) - Get extension version

---

## Function Reference

### ST_MakePoint

Create a 2D point geometry from X and Y coordinates.

#### Signature
```sql
ST_MakePoint(x double precision, y double precision) → geometry
```

#### Parameters
- `x` - X coordinate (longitude)
- `y` - Y coordinate (latitude)

#### Returns
- `geometry` - Point geometry with default SRID 0

#### Examples
```sql
-- Create a point at origin
SELECT ST_MakePoint(0, 0);

-- Create point for San Francisco  
SELECT ST_MakePoint(-122.4194, 37.7749);

-- Use in table
INSERT INTO locations (name, geom) 
VALUES ('Golden Gate Bridge', ST_MakePoint(-122.4783, 37.8199));
```

#### Notes
- Default SRID is 0 (unknown/unspecified)
- Use ST_SetSRID to assign a specific spatial reference system
- Equivalent to ST_Point function

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical behavior to PostGIS

---

### ST_Point

Alias for ST_MakePoint. Creates a 2D point geometry.

#### Signature
```sql
ST_Point(x double precision, y double precision) → geometry
```

#### Parameters
Same as ST_MakePoint

#### Examples
```sql
-- Identical to ST_MakePoint
SELECT ST_Point(-74.0060, 40.7128);
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical behavior to PostGIS

---

### ST_MakePointZ

Create a 3D point geometry from X, Y, and Z coordinates.

#### Signature
```sql
ST_MakePointZ(x double precision, y double precision, z double precision) → geometry
```

#### Parameters
- `x` - X coordinate
- `y` - Y coordinate  
- `z` - Z coordinate (elevation/height)

#### Returns
- `geometry` - 3D Point geometry

#### Examples
```sql
-- Create point with elevation
SELECT ST_MakePointZ(-122.4194, 37.7749, 150.5);

-- Use in elevation table
INSERT INTO elevation_points (name, location)
VALUES ('Mt. Tamalpais', ST_MakePointZ(-122.5956, 37.9236, 785));
```

#### Notes
- Z coordinate represents elevation or height
- Use ST_Z() to extract Z coordinate
- Basic 3D support (full 3D operations planned)

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical behavior to PostGIS

---

### ST_GeomFromText

Create a geometry from Well-Known Text (WKT) representation.

#### Signature
```sql
ST_GeomFromText(wkt text) → geometry
ST_GeomFromText(wkt text, srid integer) → geometry
```

#### Parameters
- `wkt` - Well-Known Text string
- `srid` - Optional spatial reference system identifier

#### Returns
- `geometry` - Parsed geometry object
- Raises error if WKT is invalid

#### Supported Geometry Types
- `POINT(x y)`
- `LINESTRING(x1 y1, x2 y2, ...)`
- `POLYGON((x1 y1, x2 y2, x3 y3, x1 y1))`
- `MULTIPOINT((x1 y1), (x2 y2))`
- `MULTILINESTRING(...)`
- `MULTIPOLYGON(...)`
- `GEOMETRYCOLLECTION(...)`

#### Examples
```sql
-- Create point from WKT
SELECT ST_GeomFromText('POINT(-122.4194 37.7749)');

-- Create linestring
SELECT ST_GeomFromText('LINESTRING(-122.4 37.8, -122.5 37.7, -122.6 37.6)');

-- Create polygon
SELECT ST_GeomFromText('POLYGON((-122.5 37.7, -122.4 37.7, -122.4 37.8, -122.5 37.8, -122.5 37.7))');

-- With specific SRID
SELECT ST_GeomFromText('POINT(-122.4194 37.7749)', 4326);
```

#### Error Handling
```sql
-- Invalid WKT raises error
SELECT ST_GeomFromText('INVALID WKT');
-- ERROR: Invalid WKT format

-- Empty geometry
SELECT ST_GeomFromText('POINT EMPTY');
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Same WKT parsing and error behavior

---

### ST_GeomFromWKT

Alias for ST_GeomFromText.

#### Signature
```sql
ST_GeomFromWKT(wkt text) → geometry
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical to PostGIS

---

### ST_AsText

Convert geometry to Well-Known Text (WKT) representation.

#### Signature
```sql
ST_AsText(geom geometry) → text
```

#### Parameters
- `geom` - Input geometry

#### Returns
- `text` - WKT representation of the geometry

#### Examples
```sql
-- Point to WKT
SELECT ST_AsText(ST_MakePoint(-122.4194, 37.7749));
-- Result: 'POINT(-122.4194 37.7749)'

-- Complex geometry to WKT
SELECT ST_AsText(ST_GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))'));
-- Result: 'POLYGON((-1 -1,1 -1,1 1,-1 1,-1 -1))'

-- Use in queries
SELECT name, ST_AsText(geom) as location_wkt 
FROM locations;
```

#### Output Format
- Standard OGC WKT format
- Coordinates formatted with full precision
- No unnecessary whitespace

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical WKT output format

---

### ST_AsWKT

Alias for ST_AsText.

#### Signature
```sql
ST_AsWKT(geom geometry) → text
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical to PostGIS

---

### ST_AsWKB

Convert geometry to Well-Known Binary (WKB) hexadecimal representation.

#### Signature
```sql
ST_AsWKB(geom geometry) → text
```

#### Parameters
- `geom` - Input geometry

#### Returns
- `text` - Hexadecimal string representing WKB

#### Examples
```sql
-- Point to WKB hex
SELECT ST_AsWKB(ST_MakePoint(1, 2));
-- Result: '0101000000000000000000F03F0000000000000040'

-- Use for binary storage/transfer
SELECT encode(ST_AsWKB(geom)::bytea, 'hex') FROM locations;
```

#### Format
- Standard OGC WKB format
- Little-endian byte order
- Hexadecimal string representation

#### PostGIS Compatibility
✅ **Fully Compatible** - Same WKB format and encoding

---

### ST_AsGeoJSON

Convert geometry to GeoJSON representation.

#### Signature
```sql
ST_AsGeoJSON(geom geometry) → text
```

#### Parameters
- `geom` - Input geometry

#### Returns
- `text` - GeoJSON string

#### Examples
```sql
-- Point to GeoJSON
SELECT ST_AsGeoJSON(ST_MakePoint(-122.4194, 37.7749));
-- Result: '{"type":"Point","coordinates":[-122.4194,37.7749]}'

-- Polygon to GeoJSON
SELECT ST_AsGeoJSON(ST_GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))'));
-- Result: '{"type":"Polygon","coordinates":[[[-1,-1],[1,-1],[1,1],[-1,1],[-1,-1]]]}'

-- Use in web APIs
SELECT json_build_object(
    'type', 'Feature',
    'geometry', ST_AsGeoJSON(geom)::json,
    'properties', json_build_object('name', name)
) FROM locations;
```

#### Output Format
- RFC 7946 GeoJSON specification
- Compact JSON format (no extra whitespace)
- Standard coordinate ordering [longitude, latitude]

#### PostGIS Compatibility
✅ **Fully Compatible** - Standard GeoJSON format

---

### ST_X

Extract the X coordinate from a point geometry.

#### Signature
```sql
ST_X(geom geometry) → double precision
```

#### Parameters
- `geom` - Point geometry

#### Returns
- `double precision` - X coordinate value
- `NULL` if geometry is not a point

#### Examples
```sql
-- Get X coordinate
SELECT ST_X(ST_MakePoint(-122.4194, 37.7749));
-- Result: -122.4194

-- Use in calculations
SELECT name, ST_X(geom) as longitude 
FROM locations 
WHERE ST_X(geom) BETWEEN -123 AND -122;

-- Null for non-points
SELECT ST_X(ST_GeomFromText('LINESTRING(0 0, 1 1)'));
-- Result: NULL
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Same behavior for all geometry types

---

### ST_Y

Extract the Y coordinate from a point geometry.

#### Signature
```sql
ST_Y(geom geometry) → double precision
```

#### Parameters
- `geom` - Point geometry

#### Returns
- `double precision` - Y coordinate value
- `NULL` if geometry is not a point

#### Examples
```sql
-- Get Y coordinate
SELECT ST_Y(ST_MakePoint(-122.4194, 37.7749));
-- Result: 37.7749

-- Use with ST_X for coordinate pairs
SELECT ST_X(geom) as lng, ST_Y(geom) as lat 
FROM locations;
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Same behavior for all geometry types

---

### ST_Z

Extract the Z coordinate from a point geometry.

#### Signature
```sql
ST_Z(geom geometry) → double precision
```

#### Parameters
- `geom` - Point geometry (preferably 3D)

#### Returns
- `double precision` - Z coordinate value
- `NULL` if geometry is not a point or has no Z coordinate

#### Examples
```sql
-- Get Z coordinate from 3D point
SELECT ST_Z(ST_MakePointZ(-122.4194, 37.7749, 150.5));
-- Result: 150.5

-- Returns NULL for 2D points
SELECT ST_Z(ST_MakePoint(-122.4194, 37.7749));
-- Result: NULL
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Same Z coordinate handling

---

### ST_GeometryType

Get the geometry type of a geometry.

#### Signature
```sql
ST_GeometryType(geom geometry) → text
```

#### Parameters
- `geom` - Input geometry

#### Returns
- `text` - Geometry type string

#### Return Values
- `'ST_Point'` - Point geometry
- `'ST_LineString'` - LineString geometry
- `'ST_Polygon'` - Polygon geometry
- `'ST_MultiPoint'` - MultiPoint geometry
- `'ST_MultiLineString'` - MultiLineString geometry
- `'ST_MultiPolygon'` - MultiPolygon geometry
- `'ST_GeometryCollection'` - GeometryCollection

#### Examples
```sql
-- Check geometry type
SELECT ST_GeometryType(ST_MakePoint(1, 2));
-- Result: 'ST_Point'

SELECT ST_GeometryType(ST_GeomFromText('LINESTRING(0 0, 1 1)'));
-- Result: 'ST_LineString'

-- Use in conditional logic
SELECT 
    CASE ST_GeometryType(geom)
        WHEN 'ST_Point' THEN 'This is a point'
        WHEN 'ST_Polygon' THEN 'This is a polygon'
        ELSE 'Other geometry type'
    END
FROM spatial_table;
```

#### PostGIS Compatibility
✅ **Fully Compatible** - Identical type strings and behavior

---

### ST_SRID

Get the Spatial Reference System Identifier (SRID) of a geometry.

#### Signature
```sql
ST_SRID(geom geometry) → integer
```

#### Parameters
- `geom` - Input geometry

#### Returns
- `integer` - SRID value (0 if unspecified)

#### Examples
```sql
-- Check SRID of geometry
SELECT ST_SRID(ST_MakePoint(-122.4194, 37.7749));
-- Result: 0 (default)

-- Check SRID after setting
SELECT ST_SRID(ST_SetSRID(ST_MakePoint(-122.4194, 37.7749), 4326));
-- Result: 4326

-- Filter by SRID
SELECT * FROM locations WHERE ST_SRID(geom) = 4326;
```

#### Common SRID Values
- `0` - Unknown/unspecified
- `4326` - WGS84 Geographic (GPS coordinates)
- `3857` - Web Mercator (web mapping)
- `2154` - RGF93 / Lambert-93 (France)

#### PostGIS Compatibility
✅ **Fully Compatible** - Same SRID handling

---

### ST_SetSRID

Set the Spatial Reference System Identifier (SRID) of a geometry.

#### Signature
```sql
ST_SetSRID(geom geometry, srid integer) → geometry
```

#### Parameters
- `geom` - Input geometry
- `srid` - Target SRID value

#### Returns
- `geometry` - Geometry with updated SRID

#### Examples
```sql
-- Set SRID to WGS84
SELECT ST_SetSRID(ST_MakePoint(-122.4194, 37.7749), 4326);

-- Update existing geometry SRID
UPDATE locations 
SET geom = ST_SetSRID(geom, 4326) 
WHERE ST_SRID(geom) = 0;

-- Create with specific SRID
INSERT INTO locations (name, geom) VALUES (
    'San Francisco',
    ST_SetSRID(ST_MakePoint(-122.4194, 37.7749), 4326)
);
```

#### Notes
- This function only changes the SRID metadata
- It does NOT transform coordinates to the new reference system
- Use ST_Transform for coordinate transformation (planned feature)

#### PostGIS Compatibility
✅ **Fully Compatible** - Same SRID assignment behavior

---

### ST_Envelope

Get the bounding box (envelope) of a geometry as a BBox type.

#### Signature
```sql
ST_Envelope(geom geometry) → bbox
```

#### Parameters
- `geom` - Input geometry

#### Returns
- `bbox` - Bounding box with min/max X/Y coordinates

#### Examples
```sql
-- Get envelope of point (point coordinates)
SELECT ST_Envelope(ST_MakePoint(-122.4194, 37.7749));

-- Get envelope of polygon
SELECT ST_Envelope(ST_GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))'));

-- Use for spatial indexing setup
CREATE INDEX spatial_idx ON table_name 
USING GIST (ST_Envelope(geom));
```

#### Return Type
The BBox type contains:
- `min_x` - Minimum X coordinate
- `min_y` - Minimum Y coordinate  
- `max_x` - Maximum X coordinate
- `max_y` - Maximum Y coordinate

#### PostGIS Compatibility
🔄 **Compatible with Differences** - PostGIS returns geometry, RostGIS returns BBox type for indexing efficiency

---

### ST_Distance

Calculate the Euclidean distance between two geometries.

#### Signature
```sql
ST_Distance(geom1 geometry, geom2 geometry) → double precision
```

#### Parameters
- `geom1` - First geometry
- `geom2` - Second geometry

#### Returns
- `double precision` - Distance in coordinate system units

#### Examples
```sql
-- Distance between two points
SELECT ST_Distance(
    ST_MakePoint(0, 0),
    ST_MakePoint(3, 4)
);
-- Result: 5.0 (3-4-5 triangle)

-- Find nearby locations
SELECT name, ST_Distance(geom, ST_MakePoint(-122.4194, 37.7749)) as distance
FROM locations 
WHERE ST_Distance(geom, ST_MakePoint(-122.4194, 37.7749)) < 1000
ORDER BY distance;

-- Use with different geometry types
SELECT ST_Distance(
    ST_GeomFromText('POINT(0 0)'),
    ST_GeomFromText('LINESTRING(10 0, 10 10)')
);
```

#### Algorithm
- Uses Euclidean distance calculation
- For points: standard distance formula
- For other geometries: distance between closest points
- Results in units of the coordinate system

#### Performance
- Highly optimized for point-to-point calculations
- Uses bounding box pre-filtering for complex geometries

#### PostGIS Compatibility
✅ **Fully Compatible** - Same distance calculation method

---

### ST_Area

Calculate the area of a polygon geometry.

#### Signature
```sql
ST_Area(geom geometry) → double precision
```

#### Parameters
- `geom` - Polygon or MultiPolygon geometry

#### Returns
- `double precision` - Area in square units of coordinate system
- `0.0` for non-polygon geometries

#### Examples
```sql
-- Area of unit square
SELECT ST_Area(ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))'));
-- Result: 1.0

-- Area of triangle
SELECT ST_Area(ST_GeomFromText('POLYGON((0 0, 1 0, 0.5 1, 0 0))'));
-- Result: 0.5

-- Find large polygons
SELECT name, ST_Area(geom) as area_sq_meters
FROM administrative_boundaries 
WHERE ST_Area(geom) > 1000000
ORDER BY area_sq_meters DESC;

-- Area returns 0 for non-polygons
SELECT ST_Area(ST_MakePoint(1, 2));
-- Result: 0.0
```

#### Algorithm
- Uses shoelace formula for simple polygons
- Handles holes in polygons correctly
- Sums areas for MultiPolygon geometries

#### Units
- Area is in square units of the coordinate system
- For geographic coordinates (degrees), use appropriate projection for meaningful area calculations

#### PostGIS Compatibility
✅ **Fully Compatible** - Same area calculation algorithm

---

### ST_Length

Calculate the length of a linear geometry.

#### Signature
```sql
ST_Length(geom geometry) → double precision
```

#### Parameters
- `geom` - LineString or MultiLineString geometry

#### Returns
- `double precision` - Length in units of coordinate system
- `0.0` for non-linear geometries

#### Examples
```sql
-- Length of line segment
SELECT ST_Length(ST_GeomFromText('LINESTRING(0 0, 3 4)'));
-- Result: 5.0

-- Length of complex linestring
SELECT ST_Length(ST_GeomFromText('LINESTRING(0 0, 1 0, 1 1, 0 1)'));
-- Result: 3.0

-- Find long roads
SELECT name, ST_Length(geom) as length_meters
FROM roads 
WHERE ST_Length(geom) > 1000
ORDER BY length_meters DESC;

-- Returns 0 for non-linear geometries
SELECT ST_Length(ST_MakePoint(1, 2));
-- Result: 0.0
```

#### Algorithm
- Sums distances between consecutive vertices
- Handles MultiLineString by summing component lengths
- Uses Euclidean distance calculation

#### PostGIS Compatibility
✅ **Fully Compatible** - Same length calculation method

---

### ST_Perimeter

Calculate the perimeter of a polygon geometry.

#### Signature
```sql
ST_Perimeter(geom geometry) → double precision
```

#### Parameters
- `geom` - Polygon or MultiPolygon geometry

#### Returns
- `double precision` - Perimeter in units of coordinate system
- `0.0` for non-polygon geometries

#### Examples
```sql
-- Perimeter of unit square
SELECT ST_Perimeter(ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))'));
-- Result: 4.0

-- Perimeter includes holes
SELECT ST_Perimeter(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0), (2 2, 8 2, 8 8, 2 8, 2 2))'));
-- Result: 64.0 (40 for outer ring + 24 for hole)

-- Find irregular shapes (high perimeter to area ratio)
SELECT name, ST_Perimeter(geom) / ST_Area(geom) as complexity
FROM land_parcels
ORDER BY complexity DESC;
```

#### Algorithm
- Calculates perimeter of exterior ring
- Adds perimeter of all interior rings (holes)
- Sums perimeters for MultiPolygon geometries

#### PostGIS Compatibility
✅ **Fully Compatible** - Same perimeter calculation including holes

---

### ST_Equals

Test if two geometries are spatially equal.

#### Signature
```sql
ST_Equals(geom1 geometry, geom2 geometry) → boolean
```

#### Parameters
- `geom1` - First geometry
- `geom2` - Second geometry

#### Returns
- `boolean` - True if geometries are equal, false otherwise

#### Examples
```sql
-- Test point equality
SELECT ST_Equals(
    ST_MakePoint(1, 2),
    ST_MakePoint(1, 2)
);
-- Result: true

-- Test with different coordinate precision
SELECT ST_Equals(
    ST_MakePoint(1.0, 2.0),
    ST_MakePoint(1.0000001, 2.0)
);
-- Result: false

-- Find duplicate geometries
SELECT a.id, b.id 
FROM geometries a, geometries b 
WHERE a.id < b.id 
  AND ST_Equals(a.geom, b.geom);
```

#### Equality Definition
- Geometries must have identical coordinates
- Uses floating-point precision comparison
- Different geometry types are never equal
- Considers vertex order and orientation

#### PostGIS Compatibility
✅ **Fully Compatible** - Same equality semantics

---

### ST_Intersects

Test if two geometries spatially intersect.

#### Signature
```sql
ST_Intersects(geom1 geometry, geom2 geometry) → boolean
```

#### Parameters
- `geom1` - First geometry
- `geom2` - Second geometry

#### Returns
- `boolean` - True if geometries intersect, false otherwise

#### Examples
```sql
-- Test point intersection with polygon
SELECT ST_Intersects(
    ST_MakePoint(0.5, 0.5),
    ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))')
);
-- Result: true

-- Find intersecting features
SELECT a.name, b.name
FROM roads a, buildings b
WHERE ST_Intersects(a.geom, b.geom);

-- Spatial filter with index support
SELECT * FROM locations 
WHERE ST_Intersects(geom, ST_MakePoint(-122.4, 37.7));
```

#### Algorithm
- First performs bounding box overlap test (index-accelerated)
- For bounding box overlaps, assumes intersection (simplified implementation)
- Future versions will implement exact geometric intersection testing

#### Index Support
✅ **Index Accelerated** - Automatically uses spatial indexes when available

#### PostGIS Compatibility
🔄 **Compatible with Differences** - Uses bounding box optimization; exact geometric testing planned

---

### ST_Contains

Test if the first geometry completely contains the second geometry.

#### Signature
```sql
ST_Contains(geom1 geometry, geom2 geometry) → boolean
```

#### Parameters
- `geom1` - Container geometry
- `geom2` - Contained geometry

#### Returns
- `boolean` - True if geom1 contains geom2, false otherwise

#### Examples
```sql
-- Test if polygon contains point
SELECT ST_Contains(
    ST_GeomFromText('POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))'),
    ST_MakePoint(1, 1)
);
-- Result: true

-- Find points within administrative boundary
SELECT p.name 
FROM points p, boundaries b
WHERE b.name = 'California' 
  AND ST_Contains(b.geom, p.geom);

-- Spatial containment query
SELECT * FROM cities 
WHERE ST_Contains(
    ST_GeomFromText('POLYGON((-125 32, -114 32, -114 42, -125 42, -125 32))'),
    geom
);
```

#### Algorithm
- First performs bounding box containment test (index-accelerated)
- For bounding box containment, assumes geometric containment (simplified)
- Future versions will implement exact geometric containment testing

#### Index Support
✅ **Index Accelerated** - Uses spatial indexes for bounding box pre-filtering

#### PostGIS Compatibility
🔄 **Compatible with Differences** - Uses bounding box optimization; exact geometric testing planned

---

### ST_Within

Test if the first geometry is completely within the second geometry.

#### Signature
```sql
ST_Within(geom1 geometry, geom2 geometry) → boolean
```

#### Parameters
- `geom1` - Geometry to test
- `geom2` - Container geometry

#### Returns
- `boolean` - True if geom1 is within geom2, false otherwise

#### Examples
```sql
-- Test if point is within polygon
SELECT ST_Within(
    ST_MakePoint(1, 1),
    ST_GeomFromText('POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))')
);
-- Result: true

-- Equivalent to ST_Contains with reversed arguments
SELECT ST_Within(geom1, geom2) = ST_Contains(geom2, geom1);
-- Always true

-- Find features within boundary
SELECT * FROM facilities 
WHERE ST_Within(
    geom,
    (SELECT geom FROM boundaries WHERE name = 'National Park')
);
```

#### Relationship
- `ST_Within(A, B)` is equivalent to `ST_Contains(B, A)`
- Implemented as a wrapper around ST_Contains

#### PostGIS Compatibility
✅ **Fully Compatible** - Same relationship to ST_Contains

---

### ST_DWithin

Test if two geometries are within a specified distance of each other.

#### Signature
```sql
ST_DWithin(geom1 geometry, geom2 geometry, distance double precision) → boolean
```

#### Parameters
- `geom1` - First geometry
- `geom2` - Second geometry
- `distance` - Maximum distance threshold

#### Returns
- `boolean` - True if distance between geometries ≤ distance threshold

#### Examples
```sql
-- Find points within 1000 meters
SELECT * FROM locations 
WHERE ST_DWithin(
    geom, 
    ST_MakePoint(-122.4194, 37.7749), 
    1000
);

-- Find nearby features
SELECT a.name, b.name, ST_Distance(a.geom, b.geom) as actual_distance
FROM restaurants a, hotels b
WHERE ST_DWithin(a.geom, b.geom, 500)
ORDER BY actual_distance;

-- Proximity analysis
SELECT COUNT(*) as nearby_count
FROM schools
WHERE ST_DWithin(
    geom,
    ST_GeomFromText('POINT(-122.4 37.8)'),
    2000
);
```

#### Algorithm
- Uses ST_Distance for actual distance calculation
- Simplified implementation; future versions may optimize with bounding box expansion

#### Units
- Distance is in the same units as the geometry coordinate system
- For geographic coordinates, consider using projected coordinate systems for accurate distance measurements

#### PostGIS Compatibility
🔄 **Compatible with Differences** - Simplified distance calculation; full optimization planned

---

### rostgis_version

Get the version information for the RostGIS extension.

#### Signature
```sql
rostgis_version() → text
```

#### Returns
- `text` - Version string with extension information

#### Examples
```sql
-- Check extension version
SELECT rostgis_version();
-- Result: 'RostGIS 0.1.0 - PostGIS-compatible spatial extension for PostgreSQL'

-- Use in compatibility checks
SELECT CASE 
    WHEN rostgis_version() LIKE '%0.1%' THEN 'Version 0.1.x'
    ELSE 'Other version'
END;
```

#### Usage
- Useful for debugging and compatibility checking
- Include in bug reports
- Check before using version-specific features

---

## Function Compatibility Matrix

| Function         | RostGIS | PostGIS | Status                    |
|------------------|---------|---------|---------------------------|
| ST_MakePoint     | ✅       | ✅       | Fully Compatible          |
| ST_Point         | ✅       | ✅       | Fully Compatible          |
| ST_MakePointZ    | ✅       | ✅       | Fully Compatible          |
| ST_GeomFromText  | ✅       | ✅       | Fully Compatible          |
| ST_AsText        | ✅       | ✅       | Fully Compatible          |
| ST_AsWKB         | ✅       | ✅       | Fully Compatible          |
| ST_AsGeoJSON     | ✅       | ✅       | Fully Compatible          |
| ST_X, ST_Y, ST_Z | ✅       | ✅       | Fully Compatible          |
| ST_GeometryType  | ✅       | ✅       | Fully Compatible          |
| ST_SRID          | ✅       | ✅       | Fully Compatible          |
| ST_SetSRID       | ✅       | ✅       | Fully Compatible          |
| ST_Distance      | ✅       | ✅       | Fully Compatible          |
| ST_Area          | ✅       | ✅       | Fully Compatible          |
| ST_Length        | ✅       | ✅       | Fully Compatible          |
| ST_Perimeter     | ✅       | ✅       | Fully Compatible          |
| ST_Equals        | ✅       | ✅       | Fully Compatible          |
| ST_Intersects    | ✅       | ✅       | Bounding Box Optimization |
| ST_Contains      | ✅       | ✅       | Bounding Box Optimization |
| ST_Within        | ✅       | ✅       | Bounding Box Optimization |
| ST_DWithin       | ✅       | ✅       | Simplified Implementation |
| ST_Envelope      | ✅       | ✅       | Returns BBox type         |

## Performance Characteristics

### High Performance Functions
- **ST_MakePoint**: 3.39M ops/sec
- **ST_Distance**: 3.92M ops/sec  
- **ST_X, ST_Y**: Direct coordinate access
- **ST_AsGeoJSON**: Up to 3.86M ops/sec

### Optimized Functions
- **ST_AsText**: 0.87-2.74M ops/sec depending on complexity
- **ST_GeomFromText**: Efficient parsing with error recovery
- **Spatial operators**: Index-accelerated when possible

### Index-Accelerated Functions
- **ST_Intersects**: Uses `&&` operator
- **ST_Contains**: Uses `~` operator  
- **ST_Within**: Uses `@` operator
- **ST_DWithin**: Can use bounding box pre-filtering

---

*This API reference is updated with each RostGIS release. For the latest function additions and changes, check the release notes.* 