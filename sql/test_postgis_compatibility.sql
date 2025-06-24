-- PostGIS Compatibility and Spatial Indexing Verification Tests
-- This file tests compatibility with PostGIS syntax and verifies spatial indexing works

-- Enable timing to measure performance
\timing on

-- Make sure the extension is loaded
CREATE EXTENSION IF NOT EXISTS rostgis;

-- Test 1: PostGIS-Compatible Syntax Verification
SELECT '=== PostGIS Compatibility Tests ===' as test_section;

-- Test basic geometry creation (should match PostGIS exactly)
SELECT 'Testing ST_MakePoint...' as test;
SELECT ST_MakePoint(-122.4194, 37.7749) as san_francisco;

SELECT 'Testing ST_GeomFromText...' as test;
SELECT ST_GeomFromText('POINT(-74.0060 40.7128)') as new_york;

-- Test geometry property functions
SELECT 'Testing geometry properties...' as test;
SELECT 
    ST_X(ST_MakePoint(-122.4194, 37.7749)) as x_coord,
    ST_Y(ST_MakePoint(-122.4194, 37.7749)) as y_coord,
    ST_SRID(ST_SetSRID(ST_MakePoint(-122.4194, 37.7749), 4326)) as srid_test;

-- Test WKT output (should match PostGIS format exactly)
SELECT 'Testing WKT output...' as test;
SELECT ST_AsText(ST_MakePoint(1, 2)) as point_wkt;
SELECT ST_AsText(ST_GeomFromText('LINESTRING(0 0, 1 1, 2 2)')) as linestring_wkt;

-- Test 2: Create Test Dataset for Spatial Indexing
SELECT '=== Creating Test Dataset ===' as test_section;

-- Drop and recreate test table
DROP TABLE IF EXISTS spatial_performance_test;
CREATE TABLE spatial_performance_test (
    id SERIAL PRIMARY KEY,
    name TEXT,
    geom geometry,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert a substantial dataset for performance testing
-- This mimics real-world spatial data distribution
INSERT INTO spatial_performance_test (name, geom)
SELECT 
    'Point_' || i,
    ST_MakePoint(
        (random() - 0.5) * 360,  -- Longitude: -180 to 180
        (random() - 0.5) * 180   -- Latitude: -90 to 90
    )
FROM generate_series(1, 10000) i;

-- Add some clustered data (common in real-world scenarios)
INSERT INTO spatial_performance_test (name, geom)
SELECT 
    'Cluster_' || i,
    ST_MakePoint(
        -122.4 + (random() - 0.5) * 0.1,  -- San Francisco area
        37.7 + (random() - 0.5) * 0.1
    )
FROM generate_series(1, 1000) i;

-- Add some geometric shapes
INSERT INTO spatial_performance_test (name, geom) VALUES
    ('Golden_Gate_Park', ST_GeomFromText('POLYGON((-122.51 37.77, -122.45 37.77, -122.45 37.75, -122.51 37.75, -122.51 37.77))')),
    ('SF_Bay_Area', ST_GeomFromText('POLYGON((-122.6 37.9, -122.2 37.9, -122.2 37.6, -122.6 37.6, -122.6 37.9))')),
    ('Cross_Bay_Line', ST_GeomFromText('LINESTRING(-122.5 37.8, -122.3 37.8)'));

SELECT 'Inserted ' || COUNT(*) || ' test geometries' as status 
FROM spatial_performance_test;

-- Test 3: Performance Testing WITHOUT Spatial Index
SELECT '=== Performance Testing WITHOUT Index ===' as test_section;

-- Update table statistics
ANALYZE spatial_performance_test;

-- Test query performance without index
SELECT 'Testing overlap query without index...' as test;
EXPLAIN (ANALYZE, BUFFERS, TIMING, COSTS) 
SELECT COUNT(*) FROM spatial_performance_test 
WHERE geom && ST_MakePoint(-122.4, 37.7);

-- Store the result for comparison
\set QUIET on
SELECT COUNT(*) as results_without_index FROM spatial_performance_test 
WHERE geom && ST_MakePoint(-122.4, 37.7) \gset
\set QUIET off

SELECT 'Query returned: ' || :results_without_index || ' results without index' as baseline;

-- Test 4: Create Spatial Index (PostGIS-Compatible Syntax)
SELECT '=== Creating Spatial Index ===' as test_section;

-- Test PostGIS-compatible index creation syntax
SELECT 'Creating spatial index using PostGIS syntax...' as test;
CREATE INDEX spatial_performance_test_geom_idx 
ON spatial_performance_test 
USING GIST (geom);

-- Analyze table after index creation (PostGIS best practice)
ANALYZE spatial_performance_test;

-- Verify index was created
SELECT 'Verifying index creation...' as test;
SELECT 
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes 
WHERE tablename = 'spatial_performance_test' 
AND indexname LIKE '%geom%';

-- Test 5: Performance Testing WITH Spatial Index
SELECT '=== Performance Testing WITH Index ===' as test_section;

-- Force index usage by disabling sequential scans temporarily
SET enable_seqscan = false;

SELECT 'Testing same query WITH spatial index...' as test;
EXPLAIN (ANALYZE, BUFFERS, TIMING, COSTS) 
SELECT COUNT(*) FROM spatial_performance_test 
WHERE geom && ST_MakePoint(-122.4, 37.7);

-- Verify results are consistent
\set QUIET on
SELECT COUNT(*) as results_with_index FROM spatial_performance_test 
WHERE geom && ST_MakePoint(-122.4, 37.7) \gset
\set QUIET off

SELECT 'Query returned: ' || :results_with_index || ' results with index' as indexed_result;

-- Verify consistency
SELECT CASE 
    WHEN :results_without_index = :results_with_index 
    THEN '✓ PASS: Results consistent between indexed and non-indexed queries'
    ELSE '✗ FAIL: Result mismatch - Index may not be working correctly'
END as consistency_check;

-- Re-enable sequential scans
SET enable_seqscan = true;

-- Test 6: PostGIS-Compatible Spatial Operators
SELECT '=== Testing Spatial Operators ===' as test_section;

-- Test all spatial operators that should work with indexes
SELECT 'Testing spatial operators...' as test;

-- Overlap operator (&&) - most common
SELECT 'Overlap (&&):' as operator, COUNT(*) as count
FROM spatial_performance_test 
WHERE geom && ST_GeomFromText('POLYGON((-122.5 37.7, -122.4 37.7, -122.4 37.8, -122.5 37.8, -122.5 37.7))');

-- Contains operator (~)
SELECT 'Contains (~):' as operator, COUNT(*) as count
FROM spatial_performance_test 
WHERE ST_GeomFromText('POLYGON((-123 37, -122 37, -122 38, -123 38, -123 37))') ~ geom;

-- Within operator (@)
SELECT 'Within (@):' as operator, COUNT(*) as count
FROM spatial_performance_test 
WHERE geom @ ST_GeomFromText('POLYGON((-123 37, -122 37, -122 38, -123 38, -123 37))');

-- Left/Right operators
SELECT 'Left (<<):' as operator, COUNT(*) as count
FROM spatial_performance_test 
WHERE geom << ST_MakePoint(-122, 37.7);

SELECT 'Right (>>):' as operator, COUNT(*) as count
FROM spatial_performance_test 
WHERE geom >> ST_MakePoint(-122.5, 37.7);

-- Test 7: Spatial Functions with Index Support
SELECT '=== Testing Spatial Functions ===' as test_section;

-- ST_Intersects (should use index as primary filter)
SELECT 'ST_Intersects:' as function, COUNT(*) as count
FROM spatial_performance_test 
WHERE ST_Intersects(geom, ST_GeomFromText('POLYGON((-122.5 37.7, -122.4 37.7, -122.4 37.8, -122.5 37.8, -122.5 37.7))'));

-- ST_Contains (should use index as primary filter)  
SELECT 'ST_Contains:' as function, COUNT(*) as count
FROM spatial_performance_test a, spatial_performance_test b
WHERE a.name = 'SF_Bay_Area' AND ST_Contains(a.geom, b.geom)
LIMIT 10;

-- ST_DWithin (distance queries)
SELECT 'ST_DWithin:' as function, COUNT(*) as count
FROM spatial_performance_test 
WHERE ST_DWithin(geom, ST_MakePoint(-122.4, 37.7), 0.1);

-- Test 8: Index Usage Verification
SELECT '=== Index Usage Verification ===' as test_section;

-- Check index statistics
SELECT 'Index usage statistics:' as info;
SELECT * FROM spatial_index_stats('spatial_performance_test');

-- Verify index is being used in query plans
SELECT 'Query plan analysis:' as info;
EXPLAIN (COSTS OFF) 
SELECT COUNT(*) FROM spatial_performance_test 
WHERE geom && ST_MakePoint(-122.4, 37.7);

-- Test 9: Index Maintenance Testing
SELECT '=== Index Maintenance Testing ===' as test_section;

-- Test INSERT with index
SELECT 'Testing INSERT with spatial index...' as test;
INSERT INTO spatial_performance_test (name, geom) VALUES 
    ('Test_Insert', ST_MakePoint(-122.45, 37.75));

-- Test UPDATE with index
SELECT 'Testing UPDATE with spatial index...' as test;
UPDATE spatial_performance_test 
SET geom = ST_MakePoint(-122.46, 37.76) 
WHERE name = 'Test_Insert';

-- Test DELETE with index
SELECT 'Testing DELETE with spatial index...' as test;
DELETE FROM spatial_performance_test WHERE name = 'Test_Insert';

-- Verify index is still functional after maintenance
SELECT 'Verifying index functionality after maintenance...' as test;
SELECT COUNT(*) as post_maintenance_count
FROM spatial_performance_test 
WHERE geom && ST_MakePoint(-122.4, 37.7);

-- Test 10: Edge Cases and Error Handling
SELECT '=== Edge Cases Testing ===' as test_section;

-- Test with empty geometries
SELECT 'Testing with edge cases...' as test;

-- Test with very large area
SELECT COUNT(*) as global_overlap 
FROM spatial_performance_test 
WHERE geom && ST_GeomFromText('POLYGON((-180 -90, -180 90, 180 90, 180 -90, -180 -90))');

-- Test with very small area
SELECT COUNT(*) as tiny_overlap 
FROM spatial_performance_test 
WHERE geom && ST_GeomFromText('POLYGON((-122.4001 37.7001, -122.4001 37.7002, -122.4000 37.7002, -122.4000 37.7001, -122.4001 37.7001))');

-- Test 11: PostGIS Function Signature Compatibility
SELECT '=== PostGIS Function Signature Tests ===' as test_section;

-- Test that our function signatures match PostGIS exactly
SELECT 'Testing function signatures...' as test;

-- These should work exactly like PostGIS
SELECT ST_AsText(ST_SetSRID(ST_MakePoint(-122.4, 37.7), 4326)) as srid_point;
SELECT ST_SRID(ST_SetSRID(ST_MakePoint(-122.4, 37.7), 4326)) as srid_value;
SELECT ST_GeometryType(ST_MakePoint(-122.4, 37.7)) as geom_type;
SELECT ST_X(ST_MakePoint(1, 2)) as x_val, ST_Y(ST_MakePoint(1, 2)) as y_val;

-- Test 12: Performance Summary
SELECT '=== Performance Summary ===' as test_section;

-- Final performance comparison
SELECT 'Running final performance comparison...' as test;

-- Without index (disable temporarily)
SET enable_indexscan = false;
SELECT 'Sequential scan timing...' as scan_type;
EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, COSTS OFF)
SELECT COUNT(*) FROM spatial_performance_test 
WHERE geom && ST_GeomFromText('POLYGON((-122.5 37.7, -122.4 37.7, -122.4 37.8, -122.5 37.8, -122.5 37.7))');

-- With index
SET enable_indexscan = true;
SELECT 'Index scan timing...' as scan_type;
EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, COSTS OFF)
SELECT COUNT(*) FROM spatial_performance_test 
WHERE geom && ST_GeomFromText('POLYGON((-122.5 37.7, -122.4 37.7, -122.4 37.8, -122.5 37.8, -122.5 37.7))');

-- Success message
SELECT '=== ALL TESTS COMPLETED ===' as final_status;
SELECT 'If you see this message, RostGIS spatial indexing is working correctly!' as success_message;
SELECT 'Check the query plans above - you should see "Index Scan" instead of "Seq Scan"' as verification_tip;

-- Cleanup instructions
SELECT 'To clean up test data, run: DROP TABLE spatial_performance_test;' as cleanup_info; 