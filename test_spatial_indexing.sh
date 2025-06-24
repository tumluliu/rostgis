#!/bin/bash

# Test spatial indexing functionality for RostGIS
# This script demonstrates basic spatial indexing capabilities

echo "Testing RostGIS Spatial Indexing..."

# Start PostgreSQL and connect
PGDB="rostgis_test"
PGUSER=${PGUSER:-postgres}

echo "1. Creating test database..."
dropdb --if-exists $PGDB 2>/dev/null
createdb $PGDB

echo "2. Installing RostGIS extension..."
psql $PGDB -c "CREATE EXTENSION rostgis;"

echo "3. Setting up spatial indexing..."
psql $PGDB -f sql/gist_index_setup.sql

echo "4. Creating test table..."
psql $PGDB -c "
CREATE TABLE spatial_test (
    id SERIAL PRIMARY KEY,
    name TEXT,
    geom GEOMETRY
);
"

echo "5. Inserting test data..."
psql $PGDB -c "
INSERT INTO spatial_test (name, geom) VALUES 
    ('Point A', ST_MakePoint(1, 1)),
    ('Point B', ST_MakePoint(2, 2)),
    ('Point C', ST_MakePoint(10, 10)),
    ('Point D', ST_MakePoint(1.5, 1.5)),
    ('Point E', ST_MakePoint(0, 0));
"

echo "6. Creating spatial index (this is the key test)..."
psql $PGDB -c "
CREATE INDEX spatial_test_geom_idx ON spatial_test USING GIST (geom gist_geometry_ops_simple);
"

if [ $? -eq 0 ]; then
    echo "✓ Spatial index created successfully!"
else
    echo "✗ Failed to create spatial index"
    exit 1
fi

echo "7. Testing spatial queries..."

echo "  7a. Overlap query (&&):"
psql $PGDB -c "
SELECT name, ST_AsText(geom) 
FROM spatial_test 
WHERE geom && ST_MakePoint(1.2, 1.2);
"

echo "  7b. Distance query with ST_DWithin:"
psql $PGDB -c "
SELECT name, ST_AsText(geom), ST_Distance(geom, ST_MakePoint(1, 1)) as distance
FROM spatial_test 
WHERE ST_DWithin(geom, ST_MakePoint(1, 1), 2.0)
ORDER BY distance;
"

echo "8. Checking query plans (to see if index is used)..."
psql $PGDB -c "
EXPLAIN (ANALYZE, BUFFERS) 
SELECT name 
FROM spatial_test 
WHERE geom && ST_MakePoint(1.5, 1.5);
"

echo "9. Listing all indexes..."
psql $PGDB -c "
SELECT indexname, indexdef 
FROM pg_indexes 
WHERE tablename = 'spatial_test';
"

echo ""
echo "Spatial indexing test complete!"
echo "If you see 'Index Scan using spatial_test_geom_idx' in the query plan above,"
echo "then spatial indexing is working correctly!"
echo ""
echo "You can now use spatial indexes in your applications by:"
echo "1. Creating tables with GEOMETRY columns"
echo "2. Creating indexes with: CREATE INDEX ... USING GIST (geom_column gist_geometry_ops_simple);"
echo "3. Using spatial operators like &&, <<, >>, etc. in WHERE clauses" 