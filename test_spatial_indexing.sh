#!/bin/bash

# RostGIS Spatial Indexing Test Runner
# This script tests spatial indexing functionality and PostGIS compatibility

set -e

echo "🚀 RostGIS Spatial Indexing Test Suite"
echo "======================================"

# Check if PostgreSQL is running
if ! pg_isready -q; then
    echo "❌ PostgreSQL is not running. Please start PostgreSQL first."
    exit 1
fi

# Build and install RostGIS extension
echo "📦 Building RostGIS extension..."
cargo pgrx install --release

# Create test database
TEST_DB="rostgis_test"
echo "🗄️  Creating test database: $TEST_DB"
dropdb --if-exists $TEST_DB
createdb $TEST_DB

echo "🧪 Running spatial indexing tests..."
echo "======================================="

# Run the comprehensive test suite
psql -d $TEST_DB -f sql/test_postgis_compatibility.sql

echo ""
echo "✅ Tests completed!"
echo ""
echo "📊 Key things to look for in the output above:"
echo "   • Query plans should show 'Index Scan' instead of 'Seq Scan' when using spatial operators"
echo "   • Performance should be significantly better with indexes"
echo "   • All PostGIS-compatible functions should work identically"
echo "   • Index usage statistics should show scans and reads"
echo ""
echo "🔍 To manually verify spatial indexing is working:"
echo "   1. Connect to database: psql -d $TEST_DB"
echo "   2. Run: EXPLAIN (ANALYZE, BUFFERS) SELECT COUNT(*) FROM spatial_performance_test WHERE geom && ST_MakePoint(-122.4, 37.7);"
echo "   3. Look for 'Index Scan using spatial_performance_test_geom_idx'"
echo ""
echo "🧹 To clean up: dropdb $TEST_DB" 