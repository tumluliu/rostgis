-- Test spatial indexing functionality
-- This file tests the GiST indexing support for RostGIS

-- Make sure the extension is loaded
CREATE EXTENSION IF NOT EXISTS rostgis;

-- Test 1: Basic spatial operator functionality
SELECT 'Testing basic spatial operators...' as test;

-- Create test geometries
\set point1 'ST_MakePoint(0, 0)'
\set point2 'ST_MakePoint(1, 1)'
\set point3 'ST_MakePoint(10, 10)'
\set bbox_geom 'ST_GeomFromText(''POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))'')'

-- Test bounding box operations
SELECT 
    'Overlap operator (&&)' as operation,
    (:point1) && (:point1) as same_point_overlap,
    (:point1) && (:point2) as close_points_overlap,
    (:point1) && (:point3) as distant_points_overlap;

SELECT 
    'Left operator (<<)' as operation,
    (:point1) << (:point2) as point1_left_of_point2,
    (:point2) << (:point1) as point2_left_of_point1;

SELECT 
    'Contains operator (~)' as operation,
    (:bbox_geom) ~ (:point1) as bbox_contains_point1,
    (:bbox_geom) ~ (:point3) as bbox_contains_point3;

-- Test 2: Envelope/bounding box extraction
SELECT 'Testing ST_Envelope function...' as test;

SELECT 
    'Point envelope' as geom_type,
    ST_Envelope(ST_MakePoint(5, 5)) as envelope;

SELECT 
    'Polygon envelope' as geom_type,
    ST_Envelope(ST_GeomFromText('POLYGON((1 1, 3 1, 3 3, 1 3, 1 1))')) as envelope;

-- Test 3: Create test table with spatial data
DROP TABLE IF EXISTS spatial_test_table;
CREATE TABLE spatial_test_table (
    id SERIAL PRIMARY KEY,
    name TEXT,
    geom geometry
);

-- Insert test data with various geometry types and spatial distribution
INSERT INTO spatial_test_table (name, geom) VALUES
    -- Clustered points in bottom-left
    ('Point A1', ST_MakePoint(0, 0)),
    ('Point A2', ST_MakePoint(1, 0)),
    ('Point A3', ST_MakePoint(0, 1)),
    ('Point A4', ST_MakePoint(1, 1)),
    
    -- Clustered points in top-right
    ('Point B1', ST_MakePoint(100, 100)),
    ('Point B2', ST_MakePoint(101, 100)),
    ('Point B3', ST_MakePoint(100, 101)),
    ('Point B4', ST_MakePoint(101, 101)),
    
    -- Lines
    ('Line 1', ST_GeomFromText('LINESTRING(0 0, 10 10)')),
    ('Line 2', ST_GeomFromText('LINESTRING(100 100, 110 110)')),
    
    -- Polygons
    ('Small Square', ST_GeomFromText('POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))')),
    ('Large Square', ST_GeomFromText('POLYGON((50 50, 150 50, 150 150, 50 150, 50 50))')),
    
    -- Edge cases
    ('Origin Point', ST_MakePoint(0, 0)),
    ('Negative Point', ST_MakePoint(-10, -10));

-- Test 4: Query performance without index (sequential scan)
SELECT 'Testing queries without spatial index (baseline)...' as test;

-- Time a spatial query without index
EXPLAIN (ANALYZE, BUFFERS, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT name FROM spatial_test_table 
WHERE geom && ST_MakePoint(0.5, 0.5);

-- Test 5: Create spatial index
SELECT 'Creating spatial index...' as test;
SELECT create_spatial_index('spatial_test_table', 'geom');

-- Verify index was created
SELECT 
    'Index verification' as test,
    has_spatial_index('spatial_test_table', 'geom') as index_exists;

-- Test 6: Query performance with index
SELECT 'Testing queries with spatial index...' as test;

-- Force PostgreSQL to use index for comparison
SET enable_seqscan = false;

-- Time the same spatial query with index
EXPLAIN (ANALYZE, BUFFERS, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT name FROM spatial_test_table 
WHERE geom && ST_MakePoint(0.5, 0.5);

-- Reset seqscan setting
SET enable_seqscan = true;

-- Test 7: Various spatial queries that should use the index
SELECT 'Testing various spatial queries...' as test;

-- Overlap queries
SELECT 'Overlap query results:' as query_type;
SELECT name FROM spatial_test_table 
WHERE geom && ST_GeomFromText('POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))');

-- Containment queries
SELECT 'Containment query results:' as query_type;
SELECT name FROM spatial_test_table 
WHERE ST_GeomFromText('POLYGON((50 50, 150 50, 150 150, 50 150, 50 150))') ~ geom;

-- Distance queries
SELECT 'Distance query results:' as query_type;
SELECT name FROM spatial_test_table 
WHERE ST_DWithin(geom, ST_MakePoint(0, 0), 2);

-- Test 8: Join queries with spatial predicates
SELECT 'Testing spatial joins...' as test;

-- Self-join to find overlapping geometries
SELECT 
    a.name as geom1,
    b.name as geom2
FROM spatial_test_table a, spatial_test_table b
WHERE a.id < b.id AND a.geom && b.geom
LIMIT 5;

-- Test 9: Index statistics and usage
SELECT 'Checking index statistics...' as test;
SELECT * FROM spatial_index_stats('spatial_test_table');

-- Test 10: Stress test with larger dataset
SELECT 'Creating larger dataset for stress test...' as test;

-- Create a larger test table
DROP TABLE IF EXISTS spatial_stress_test;
CREATE TABLE spatial_stress_test (
    id SERIAL PRIMARY KEY,
    geom geometry
);

-- Insert 1000 random points
INSERT INTO spatial_stress_test (geom)
SELECT ST_MakePoint(
    random() * 1000,  -- X coordinate 0-1000
    random() * 1000   -- Y coordinate 0-1000
) FROM generate_series(1, 1000);

-- Create index on stress test table
SELECT create_spatial_index('spatial_stress_test', 'geom');

-- Test query performance on larger dataset
SELECT 'Testing query on larger dataset...' as test;

-- Query that should return about 1% of data (10x10 square out of 1000x1000)
EXPLAIN (ANALYZE, BUFFERS, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT COUNT(*) FROM spatial_stress_test 
WHERE geom && ST_GeomFromText('POLYGON((100 100, 110 100, 110 110, 100 110, 100 100))');

-- Get actual count
SELECT 
    'Points in test area' as description,
    COUNT(*) as count
FROM spatial_stress_test 
WHERE geom && ST_GeomFromText('POLYGON((100 100, 110 100, 110 110, 100 110, 100 100))');

-- Test 11: Index maintenance (INSERT/UPDATE/DELETE)
SELECT 'Testing index maintenance...' as test;

-- Insert new geometry
INSERT INTO spatial_test_table (name, geom) VALUES 
    ('New Point', ST_MakePoint(50, 50));

-- Update existing geometry  
UPDATE spatial_test_table 
SET geom = ST_MakePoint(60, 60) 
WHERE name = 'New Point';

-- Delete geometry
DELETE FROM spatial_test_table WHERE name = 'New Point';

-- Verify index is still functional
SELECT COUNT(*) as post_maintenance_count 
FROM spatial_test_table 
WHERE geom && ST_MakePoint(0.5, 0.5);

-- Test 12: Verify spatial function accuracy
SELECT 'Testing spatial function accuracy...' as test;

-- Test that bounding box operations are consistent
SELECT 
    a.name,
    b.name,
    a.geom && b.geom as bbox_overlap,
    ST_Intersects(a.geom, b.geom) as actual_intersect
FROM spatial_test_table a, spatial_test_table b
WHERE a.id = 1 AND b.id IN (2, 3, 4)
ORDER BY b.id;

-- Final cleanup for stress test
DROP TABLE IF EXISTS spatial_stress_test;

SELECT 'Spatial indexing tests completed!' as test; 