-- RostGIS vs PostGIS Performance Benchmark Suite
-- This script generates comprehensive performance comparison data
-- Run this in a database with both PostGIS and RostGIS installed

-- Enable timing and detailed output
\timing on
\echo '=========================================='
\echo 'RostGIS vs PostGIS Performance Benchmark'
\echo 'Without Spatial Indexing'
\echo '=========================================='

-- Check extension availability
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis') THEN
        RAISE NOTICE 'PostGIS not found. Some comparisons will be skipped.';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'rostgis') THEN
        RAISE EXCEPTION 'RostGIS extension required for benchmarking.';
    END IF;
END $$;

-- Create benchmark tracking table
DROP TABLE IF EXISTS benchmark_results;
CREATE TABLE benchmark_results (
    test_name TEXT,
    implementation TEXT,
    execution_time_ms NUMERIC,
    operations_per_second NUMERIC,
    memory_usage_mb NUMERIC,
    test_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Clean up any existing test tables
DROP TABLE IF EXISTS rostgis_points, postgis_points;
DROP TABLE IF EXISTS rostgis_bulk_test, postgis_bulk_test;

\echo ''
\echo '=== TEST 1: Point Creation Performance ==='

-- RostGIS Point Creation Test
\echo 'Testing RostGIS point creation...'
CREATE TABLE rostgis_points (id SERIAL PRIMARY KEY, geom geometry);

\timing on
INSERT INTO rostgis_points (geom)
SELECT ST_MakePoint(random() * 100, random() * 100)
FROM generate_series(1, 10000);
\timing off

-- Measure actual performance
DO $$
DECLARE
    start_time TIMESTAMP;
    end_time TIMESTAMP;
    duration NUMERIC;
BEGIN
    start_time := clock_timestamp();
    
    PERFORM ST_MakePoint(random() * 100, random() * 100)
    FROM generate_series(1, 100000);
    
    end_time := clock_timestamp();
    duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
    
    INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
    VALUES ('point_creation_100k', 'rostgis', duration, 100000.0 / (duration / 1000.0));
END $$;

-- PostGIS Point Creation Test (if available)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis') THEN
        RAISE NOTICE 'Testing PostGIS point creation...';
        
        EXECUTE 'CREATE TABLE postgis_points (id SERIAL PRIMARY KEY, geom geometry)';
        
        DECLARE
            start_time TIMESTAMP;
            end_time TIMESTAMP;
            duration NUMERIC;
        BEGIN
            start_time := clock_timestamp();
            
            EXECUTE 'INSERT INTO postgis_points (geom)
                     SELECT ST_MakePoint(random() * 100, random() * 100)
                     FROM generate_series(1, 100000)';
            
            end_time := clock_timestamp();
            duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
            
            INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
            VALUES ('point_creation_100k', 'postgis', duration, 100000.0 / (duration / 1000.0));
        END;
    END IF;
END $$;

\echo ''
\echo '=== TEST 2: WKT Parsing Performance ==='

-- Test WKT Parsing Performance
DO $$
DECLARE
    start_time TIMESTAMP;
    end_time TIMESTAMP;
    duration NUMERIC;
    test_wkt TEXT[] := ARRAY[
        'POINT(1 2)',
        'LINESTRING(0 0, 1 1, 2 2, 3 3)',
        'POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))'
    ];
    wkt_type TEXT[] := ARRAY['point', 'linestring', 'polygon'];
    i INTEGER;
BEGIN
    FOR i IN 1..3 LOOP
        -- RostGIS WKT Parsing
        start_time := clock_timestamp();
        
        PERFORM ST_GeomFromText(test_wkt[i])
        FROM generate_series(1, 50000);
        
        end_time := clock_timestamp();
        duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
        
        INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
        VALUES ('wkt_parsing_' || wkt_type[i], 'rostgis', duration, 50000.0 / (duration / 1000.0));
        
        -- PostGIS WKT Parsing (if available)
        IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis') THEN
            start_time := clock_timestamp();
            
            EXECUTE 'SELECT ST_GeomFromText($1) FROM generate_series(1, 50000)' USING test_wkt[i];
            
            end_time := clock_timestamp();
            duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
            
            INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
            VALUES ('wkt_parsing_' || wkt_type[i], 'postgis', duration, 50000.0 / (duration / 1000.0));
        END IF;
    END LOOP;
END $$;

\echo ''
\echo '=== TEST 3: Distance Calculation Performance ==='

DO $$
DECLARE
    start_time TIMESTAMP;
    end_time TIMESTAMP;
    duration NUMERIC;
    point1 geometry := ST_MakePoint(0, 0);
    point2 geometry := ST_MakePoint(3, 4);
BEGIN
    -- RostGIS Distance Calculation
    start_time := clock_timestamp();
    
    PERFORM ST_Distance(point1, point2)
    FROM generate_series(1, 100000);
    
    end_time := clock_timestamp();
    duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
    
    INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
    VALUES ('distance_calculation_100k', 'rostgis', duration, 100000.0 / (duration / 1000.0));
    
    -- PostGIS comparison would go here if available
END $$;

\echo ''
\echo '=== TEST 4: Sequential Scan Performance (No Index) ==='

-- Create larger dataset for scanning tests
DROP TABLE IF EXISTS scan_test_rostgis;
CREATE TABLE scan_test_rostgis (
    id SERIAL PRIMARY KEY,
    geom geometry,
    category TEXT
);

INSERT INTO scan_test_rostgis (geom, category)
SELECT 
    ST_MakePoint(random() * 360 - 180, random() * 180 - 90),
    CASE (random() * 4)::INTEGER
        WHEN 0 THEN 'urban'
        WHEN 1 THEN 'rural'
        WHEN 2 THEN 'industrial'
        ELSE 'residential'
    END
FROM generate_series(1, 50000);

-- Test bounding box overlap queries
\echo 'Testing spatial overlap queries...'

EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, COSTS OFF)
SELECT COUNT(*) FROM scan_test_rostgis 
WHERE geom && ST_MakePoint(-122, 37);

-- Test spatial relationship queries
EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, COSTS OFF)
SELECT COUNT(*) FROM scan_test_rostgis a, scan_test_rostgis b
WHERE a.id < b.id AND ST_DWithin(a.geom, b.geom, 1.0)
LIMIT 1000;

\echo ''
\echo '=== TEST 5: Bulk Operations Performance ==='

-- Test bulk insertion performance
DO $$
DECLARE
    start_time TIMESTAMP;
    end_time TIMESTAMP;
    duration NUMERIC;
BEGIN
    -- RostGIS Bulk Insert
    DROP TABLE IF EXISTS bulk_insert_rostgis;
    CREATE TABLE bulk_insert_rostgis (id SERIAL PRIMARY KEY, geom geometry);
    
    start_time := clock_timestamp();
    
    INSERT INTO bulk_insert_rostgis (geom)
    SELECT ST_MakePoint(random() * 100, random() * 100)
    FROM generate_series(1, 25000);
    
    end_time := clock_timestamp();
    duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
    
    INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
    VALUES ('bulk_insert_25k', 'rostgis', duration, 25000.0 / (duration / 1000.0));
END $$;

\echo ''
\echo '=== TEST 6: GeoJSON Serialization Performance ==='

DO $$
DECLARE
    start_time TIMESTAMP;
    end_time TIMESTAMP;
    duration NUMERIC;
    test_geoms geometry[] := ARRAY[
        ST_MakePoint(1, 2),
        ST_GeomFromText('LINESTRING(0 0, 1 1, 2 2)'),
        ST_GeomFromText('POLYGON((0 0, 5 0, 5 5, 0 5, 0 0))')
    ];
    geom_types TEXT[] := ARRAY['point', 'linestring', 'polygon'];
    i INTEGER;
BEGIN
    FOR i IN 1..3 LOOP
        start_time := clock_timestamp();
        
        PERFORM ST_AsGeoJSON(test_geoms[i])
        FROM generate_series(1, 25000);
        
        end_time := clock_timestamp();
        duration := EXTRACT(EPOCH FROM (end_time - start_time)) * 1000;
        
        INSERT INTO benchmark_results (test_name, implementation, execution_time_ms, operations_per_second)
        VALUES ('geojson_' || geom_types[i], 'rostgis', duration, 25000.0 / (duration / 1000.0));
    END LOOP;
END $$;

\echo ''
\echo '=== TEST 7: Memory Usage Analysis ==='

-- Check table sizes and memory usage
SELECT 
    schemaname,
    tablename,
    attname,
    n_distinct,
    correlation,
    most_common_vals
FROM pg_stats 
WHERE tablename LIKE '%rostgis%' OR tablename LIKE '%postgis%';

-- Table size comparison
SELECT 
    tablename,
    pg_size_pretty(pg_total_relation_size(tablename::regclass)) as total_size,
    pg_size_pretty(pg_relation_size(tablename::regclass)) as table_size,
    pg_size_pretty(pg_total_relation_size(tablename::regclass) - pg_relation_size(tablename::regclass)) as index_size
FROM (
    SELECT 'rostgis_points' as tablename
    UNION ALL
    SELECT 'scan_test_rostgis'
    UNION ALL
    SELECT 'bulk_insert_rostgis'
) t
WHERE EXISTS (SELECT 1 FROM pg_tables WHERE tablename = t.tablename);

\echo ''
\echo '=== BENCHMARK RESULTS SUMMARY ==='

-- Display comprehensive results
SELECT 
    test_name,
    implementation,
    ROUND(execution_time_ms, 2) as exec_time_ms,
    ROUND(operations_per_second, 0) as ops_per_sec,
    CASE 
        WHEN LAG(operations_per_second) OVER (PARTITION BY test_name ORDER BY implementation) IS NOT NULL
        THEN ROUND(((operations_per_second / LAG(operations_per_second) OVER (PARTITION BY test_name ORDER BY implementation)) - 1) * 100, 1)
        ELSE NULL
    END as improvement_pct
FROM benchmark_results
ORDER BY test_name, implementation;

-- Performance comparison matrix
WITH performance_matrix AS (
    SELECT 
        test_name,
        MAX(CASE WHEN implementation = 'rostgis' THEN operations_per_second END) as rostgis_ops,
        MAX(CASE WHEN implementation = 'postgis' THEN operations_per_second END) as postgis_ops
    FROM benchmark_results
    GROUP BY test_name
)
SELECT 
    test_name,
    COALESCE(ROUND(rostgis_ops, 0), 0) as rostgis_ops_sec,
    COALESCE(ROUND(postgis_ops, 0), 0) as postgis_ops_sec,
    CASE 
        WHEN postgis_ops IS NOT NULL AND postgis_ops > 0 
        THEN ROUND(((rostgis_ops / postgis_ops) - 1) * 100, 1) || '%'
        ELSE 'N/A'
    END as rostgis_improvement
FROM performance_matrix
ORDER BY test_name;

-- Generate ASCII bar charts for key metrics
\echo ''
\echo '=== PERFORMANCE VISUALIZATION ==='

-- Point Creation Performance Chart
WITH point_perf AS (
    SELECT 
        implementation,
        operations_per_second,
        ROUND(operations_per_second / 1000, 0) as ops_k
    FROM benchmark_results 
    WHERE test_name = 'point_creation_100k'
)
SELECT 
    implementation,
    ops_k || 'K ops/sec' as performance,
    REPEAT('█', (ops_k::INTEGER / 50)) as bar_chart
FROM point_perf
ORDER BY ops_k DESC;

-- Memory efficiency summary
\echo ''
\echo '=== MEMORY EFFICIENCY REPORT ==='

SELECT 
    'Database Size' as metric,
    pg_size_pretty(pg_database_size(current_database())) as value;

-- Test data cleanup options
\echo ''
\echo '=== CLEANUP OPTIONS ==='
\echo 'To clean up test data, run:'
\echo 'DROP TABLE IF EXISTS rostgis_points, postgis_points, scan_test_rostgis, bulk_insert_rostgis, benchmark_results;'

\echo ''
\echo '=========================================='
\echo 'Benchmark Complete!'
\echo 'Results saved in benchmark_results table'
\echo '==========================================' 