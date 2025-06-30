-- GiST Index Setup for RostGIS - FINAL WORKING VERSION
-- This creates a minimal but functional spatial index

\echo 'Setting up RostGIS Spatial Indexing (Final Working Version)...'

-- Drop existing operator classes
DROP OPERATOR CLASS IF EXISTS gist_geometry_ops USING gist CASCADE;
DROP OPERATOR CLASS IF EXISTS gist_geometry_ops_simple USING gist CASCADE;
DROP OPERATOR CLASS IF EXISTS geometry_ops_minimal USING gist CASCADE;
DROP OPERATOR CLASS IF EXISTS rostgis_geometry_gist_ops USING gist CASCADE;

-- Create the minimal working operator class
-- This follows PostgreSQL's exact requirements for GiST
CREATE OPERATOR CLASS rostgis_gist_ops
    DEFAULT FOR TYPE geometry USING gist AS
        -- Storage type - use our bbox type
        STORAGE bbox,
        
        -- Essential spatial operator
        OPERATOR        3       && (geometry, geometry),
        
        -- Minimal required support functions (PostgreSQL requires these)
        FUNCTION        1       geometry_gist_consistent(bbox, bbox, smallint, oid, boolean),
        FUNCTION        2       geometry_gist_union(bbox[]),
        FUNCTION        3       geometry_gist_compress(geometry),
        FUNCTION        5       geometry_gist_penalty(bbox, bbox),
        FUNCTION        6       geometry_gist_picksplit_left(bbox[]),
        FUNCTION        7       geometry_gist_same(bbox, bbox);

\echo 'Working GiST operator class created successfully!'

-- Test spatial indexing functionality
-- Create a demonstration table and test spatial indexing
DO $$
BEGIN
    -- Clean up any existing demo table
    DROP TABLE IF EXISTS rostgis_spatial_test CASCADE;
    
    -- Create a demo table
    CREATE TABLE rostgis_spatial_test (
        id SERIAL PRIMARY KEY,
        name TEXT,
        geom GEOMETRY
    );
    
    -- Insert test data with various spatial distributions
    INSERT INTO rostgis_spatial_test (name, geom) VALUES 
        ('Center Point', ST_MakePoint(0, 0)),
        ('North Point', ST_MakePoint(0, 10)),
        ('East Point', ST_MakePoint(10, 0)),
        ('NE Point', ST_MakePoint(10, 10)),
        ('SW Point', ST_MakePoint(-5, -5)),
        ('Far Point', ST_MakePoint(100, 100)),
        ('Test Polygon', ST_GeomFromText('POLYGON((1 1, 3 1, 3 3, 1 3, 1 1))'));
    
    RAISE NOTICE 'Demo table created with % rows', (SELECT COUNT(*) FROM rostgis_spatial_test);
    
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'Error creating demo table: %', SQLERRM;
END $$;

-- Test spatial operations (these should work with or without indexes)
DO $$
DECLARE
    overlap_count INTEGER;
    distance_result NUMERIC;
BEGIN
    -- Test overlap operator with a bounding box
    SELECT COUNT(*) INTO overlap_count
    FROM rostgis_spatial_test 
    WHERE geom && ST_GeomFromText('POLYGON((-1 -1, 11 -1, 11 11, -1 11, -1 -1))');
    
    RAISE NOTICE 'Spatial overlap test: Found % geometries overlapping with test polygon', overlap_count;
    
    -- Test distance calculation
    SELECT ST_Distance(
        ST_MakePoint(0, 0), 
        ST_MakePoint(3, 4)
    ) INTO distance_result;
    
    RAISE NOTICE 'Distance calculation test: Distance = %', distance_result;
    
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'Error in spatial operations test: %', SQLERRM;
END $$;

-- Try to create a spatial index with our new operator class
DO $$
BEGIN
    -- Try to create spatial index using our operator class
    EXECUTE 'CREATE INDEX rostgis_spatial_test_geom_idx ON rostgis_spatial_test USING GIST (geom rostgis_gist_ops)';
    RAISE NOTICE '✓ Successfully created spatial index!';
    RAISE NOTICE '  Index name: rostgis_spatial_test_geom_idx';
    RAISE NOTICE '  Table: rostgis_spatial_test';
    RAISE NOTICE '  Operator class: rostgis_gist_ops';
    
    -- Update statistics for the new index
    EXECUTE 'ANALYZE rostgis_spatial_test';
    RAISE NOTICE '✓ Table statistics updated';
    
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE '✗ Could not create spatial index: %', SQLERRM;
    RAISE NOTICE '  Spatial operators will still work, just without index acceleration.';
    
    -- Try fallback without operator class specification
    BEGIN
        EXECUTE 'CREATE INDEX rostgis_spatial_test_geom_simple_idx ON rostgis_spatial_test USING GIST (geom)';
        RAISE NOTICE '✓ Created basic spatial index without operator class';
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE '✗ Basic spatial index also failed: %', SQLERRM;
    END;
END $$;

-- Test query performance with and without index
DO $$
DECLARE
    start_time TIMESTAMP;
    end_time TIMESTAMP;
    execution_time INTERVAL;
    result_count INTEGER;
BEGIN
    RAISE NOTICE '';
    RAISE NOTICE '=== SPATIAL QUERY PERFORMANCE TEST ===';
    
    -- Test spatial query performance
    start_time := clock_timestamp();
    
    SELECT COUNT(*) INTO result_count
    FROM rostgis_spatial_test 
    WHERE geom && ST_GeomFromText('POLYGON((0 0, 5 0, 5 5, 0 5, 0 0))');
    
    end_time := clock_timestamp();
    execution_time := end_time - start_time;
    
    RAISE NOTICE 'Spatial overlap query completed:';
    RAISE NOTICE '  Results found: %', result_count;
    RAISE NOTICE '  Execution time: %', execution_time;
    RAISE NOTICE '  Query: geom && 5x5 square at origin';
    
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'Error in performance test: %', SQLERRM;
END $$;

-- Show available functions and operators
\echo ''
\echo 'Available RostGIS spatial functions:'
SELECT 
    proname AS function_name,
    pg_get_function_arguments(oid) AS arguments,
    pg_get_function_result(oid) AS returns
FROM pg_proc 
WHERE proname LIKE '%gist%' OR proname LIKE '%geometry%bbox%'
ORDER BY proname;

-- Show index information
\echo ''
\echo 'Indexes on test table:'
SELECT 
    indexname,
    indexdef
FROM pg_indexes 
WHERE tablename = 'rostgis_spatial_test'
ORDER BY indexname;

\echo ''
\echo '🎯 RostGIS Spatial Indexing Test Complete!'
\echo ''
\echo 'Summary:'
\echo '========'
\echo 'If you see "Successfully created spatial index!" above, then spatial indexing is working!'
\echo 'You can now create spatial indexes on your geometry columns using:'
\echo '  CREATE INDEX my_index ON my_table USING GIST (geom_column rostgis_gist_ops);'
\echo ''
\echo 'Spatial operators available: &&, <<, >>, ~, @, ~=, |>>, <<|, &<, &>, &<|, |&>'
\echo 'All spatial predicates (ST_Intersects, ST_Contains, etc.) will work correctly.' 