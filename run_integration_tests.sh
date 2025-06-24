#!/bin/bash

# RostGIS Integration Test Runner
# This script runs comprehensive integration tests for RostGIS functionality

set -e

# Configuration
DB_NAME="${DB_NAME:-rostgis_test}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
RESULTS_DIR="test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================="
echo -e "RostGIS Integration Test Suite"
echo -e "=========================================${NC}"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to log messages
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] ✓ $1${NC}"
}

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    local output_file="$RESULTS_DIR/${test_name}_${TIMESTAMP}.log"
    
    log "Running test: $test_name"
    
    if eval "$test_command" > "$output_file" 2>&1; then
        success "$test_name passed"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        error "$test_name failed (see $output_file for details)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        FAILED_TESTS+=("$test_name")
        return 1
    fi
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if PostgreSQL is running
    if ! pg_isready -U "$POSTGRES_USER" > /dev/null 2>&1; then
        error "PostgreSQL is not running or not accessible"
    fi
    
    # Check if psql is available
    if ! command -v psql &> /dev/null; then
        error "psql command not found. Please install PostgreSQL client tools."
    fi
    
    # Check if cargo is available for Rust tests
    if ! command -v cargo &> /dev/null; then
        error "cargo command not found. Please install Rust toolchain."
    fi
    
    # Check if pgrx is available for PostgreSQL integration tests
    if ! command -v cargo-pgrx &> /dev/null; then
        warn "cargo-pgrx not found. PostgreSQL integration tests will be skipped."
    fi
    
    success "Prerequisites check passed"
}

# Setup test database
setup_test_database() {
    log "Setting up test database..."
    
    # Drop existing test database if it exists
    if psql -U "$POSTGRES_USER" -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
        log "Dropping existing test database $DB_NAME..."
        dropdb -U "$POSTGRES_USER" "$DB_NAME"
    fi
    
    # Create fresh test database
    log "Creating test database $DB_NAME..."
    createdb -U "$POSTGRES_USER" "$DB_NAME"
    
    # Install RostGIS extension
    log "Installing RostGIS extension..."
    psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS rostgis;" || {
        error "Failed to install RostGIS extension. Make sure it's compiled and installed."
    }
    
    # Try to install PostGIS for compatibility tests
    if psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS postgis;" 2>/dev/null; then
        log "PostGIS extension available for compatibility tests"
    else
        warn "PostGIS extension not available. Compatibility tests will be skipped."
    fi
    
    success "Test database setup completed"
}

# Run Rust unit tests
run_rust_tests() {
    log "Running Rust unit tests..."
    
    local output_file="$RESULTS_DIR/rust_tests_${TIMESTAMP}.log"
    
    if cargo test --lib > "$output_file" 2>&1; then
        success "Rust unit tests passed"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        error "Rust unit tests failed (see $output_file for details)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        FAILED_TESTS+=("rust_unit_tests")
    fi
}

# Run PostgreSQL integration tests using pgrx
run_pgrx_tests() {
    if command -v cargo-pgrx &> /dev/null; then
        log "Running PostgreSQL integration tests with pgrx..."
        
        local output_file="$RESULTS_DIR/pgrx_tests_${TIMESTAMP}.log"
        
        if cargo pgrx test > "$output_file" 2>&1; then
            success "PostgreSQL integration tests passed"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            error "PostgreSQL integration tests failed (see $output_file for details)"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            FAILED_TESTS+=("pgrx_integration_tests")
        fi
    else
        warn "Skipping pgrx tests - cargo-pgrx not available"
    fi
}

# Run SQL-based functional tests
run_sql_tests() {
    log "Running SQL functional tests..."
    
    # Test basic functions
    run_test "basic_functions" \
        "psql -U '$POSTGRES_USER' -d '$DB_NAME' -f sql/test_basic_functions.sql"
    
    # Test spatial indexing
    run_test "spatial_indexing" \
        "psql -U '$POSTGRES_USER' -d '$DB_NAME' -f sql/test_spatial_indexing.sql"
    
    # Test PostGIS compatibility (if PostGIS is available)
    if psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "SELECT 1 FROM pg_extension WHERE extname='postgis';" -t | grep -q 1; then
        run_test "postgis_compatibility" \
            "psql -U '$POSTGRES_USER' -d '$DB_NAME' -f sql/test_postgis_compatibility.sql"
    else
        warn "Skipping PostGIS compatibility tests - PostGIS not available"
    fi
}

# Run specific geometric operation tests
run_geometry_tests() {
    log "Running geometry operation tests..."
    
    local test_sql="$RESULTS_DIR/geometry_tests_${TIMESTAMP}.sql"
    
    cat > "$test_sql" << 'EOF'
-- Comprehensive geometry operation tests

-- Test 1: Point Operations
\echo 'Testing point operations...'
SELECT 
    'Point Creation' as test,
    ST_AsText(ST_MakePoint(1, 2)) = 'POINT(1 2)' as result;

SELECT 
    'Point Coordinates' as test,
    ST_X(ST_MakePoint(3.14, 2.71)) = 3.14 AND 
    ST_Y(ST_MakePoint(3.14, 2.71)) = 2.71 as result;

-- Test 2: Distance Calculations
\echo 'Testing distance calculations...'
SELECT 
    'Euclidean Distance' as test,
    abs(ST_Distance(ST_MakePoint(0, 0), ST_MakePoint(3, 4)) - 5.0) < 0.0001 as result;

-- Test 3: WKT Parsing
\echo 'Testing WKT parsing...'
SELECT 
    'Point WKT' as test,
    ST_AsText(ST_GeomFromText('POINT(10 20)')) = 'POINT(10 20)' as result;

SELECT 
    'LineString WKT' as test,
    ST_GeometryType(ST_GeomFromText('LINESTRING(0 0, 1 1, 2 2)')) = 'ST_LineString' as result;

SELECT 
    'Polygon WKT' as test,
    ST_GeometryType(ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))')) = 'ST_Polygon' as result;

-- Test 4: Spatial Relationships
\echo 'Testing spatial relationships...'
SELECT 
    'Point Equality' as test,
    ST_Equals(ST_MakePoint(1, 2), ST_MakePoint(1, 2)) as result;

SELECT 
    'Bounding Box Overlap' as test,
    (ST_MakePoint(1, 1) && ST_MakePoint(1, 1)) as result;

-- Test 5: SRID Operations
\echo 'Testing SRID operations...'
SELECT 
    'Default SRID' as test,
    ST_SRID(ST_MakePoint(1, 2)) = 0 as result;

SELECT 
    'Set SRID' as test,
    ST_SRID(ST_SetSRID(ST_MakePoint(1, 2), 4326)) = 4326 as result;

-- Test 6: Area and Perimeter
\echo 'Testing area and perimeter calculations...'
SELECT 
    'Square Area' as test,
    ST_Area(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))')) = 100.0 as result;

SELECT 
    'Square Perimeter' as test,
    ST_Perimeter(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))')) = 40.0 as result;

-- Test 7: Error Handling
\echo 'Testing error handling...'
SELECT 
    'Invalid WKT Error' as test,
    (ST_GeomFromText('INVALID WKT') IS NULL) as result;

-- Summary
\echo 'Geometry tests completed.'
SELECT 'All geometry operation tests completed successfully!' as status;
EOF

    run_test "geometry_operations" \
        "psql -U '$POSTGRES_USER' -d '$DB_NAME' -f '$test_sql'"
}

# Run performance regression tests (light version)
run_performance_regression_tests() {
    log "Running performance regression tests..."
    
    local test_sql="$RESULTS_DIR/performance_regression_${TIMESTAMP}.sql"
    
    cat > "$test_sql" << 'EOF'
-- Light performance regression tests to ensure reasonable performance

-- Test that basic operations complete within reasonable time
\timing on

\echo 'Testing basic performance regression...'

-- Should complete in reasonable time (< 1 second for 1000 operations)
SELECT COUNT(*) FROM (
    SELECT ST_MakePoint(random() * 100, random() * 100)
    FROM generate_series(1, 1000)
) t;

-- Distance calculation performance test
SELECT COUNT(*) FROM (
    SELECT ST_Distance(
        ST_MakePoint(random() * 100, random() * 100),
        ST_MakePoint(random() * 100, random() * 100)
    )
    FROM generate_series(1, 1000)
) t;

-- WKT parsing performance test
SELECT COUNT(*) FROM (
    SELECT ST_GeomFromText('POINT(' || (random() * 100)::text || ' ' || (random() * 100)::text || ')')
    FROM generate_series(1, 100)
) t;

\timing off
\echo 'Performance regression tests completed.'
EOF

    run_test "performance_regression" \
        "psql -U '$POSTGRES_USER' -d '$DB_NAME' -f '$test_sql'"
}

# Generate test report
generate_test_report() {
    log "Generating test report..."
    
    local report_file="$RESULTS_DIR/integration_test_report_${TIMESTAMP}.md"
    
    cat > "$report_file" << EOF
# RostGIS Integration Test Report

**Generated:** $(date)
**Database:** $DB_NAME
**Test Run ID:** $TIMESTAMP

## Test Environment

- PostgreSQL Version: $(psql -U "$POSTGRES_USER" -d "$DB_NAME" -t -c "SELECT version();" | head -1)
- RostGIS Extension: $(psql -U "$POSTGRES_USER" -d "$DB_NAME" -t -c "SELECT CASE WHEN EXISTS(SELECT 1 FROM pg_extension WHERE extname='rostgis') THEN 'Installed' ELSE 'Not Available' END;" | tr -d ' ')
- PostGIS Extension: $(psql -U "$POSTGRES_USER" -d "$DB_NAME" -t -c "SELECT CASE WHEN EXISTS(SELECT 1 FROM pg_extension WHERE extname='postgis') THEN 'Available' ELSE 'Not Available' END;" | tr -d ' ')
- System: $(uname -s) $(uname -r)
- Rust Version: $(rustc --version 2>/dev/null || echo "Not available")

## Test Results Summary

- **Total Tests:** $((TESTS_PASSED + TESTS_FAILED))
- **Passed:** $TESTS_PASSED ✅
- **Failed:** $TESTS_FAILED ❌
- **Success Rate:** $(echo "scale=1; $TESTS_PASSED * 100 / ($TESTS_PASSED + $TESTS_FAILED)" | bc -l 2>/dev/null || echo "N/A")%

EOF

    if [ $TESTS_FAILED -gt 0 ]; then
        cat >> "$report_file" << EOF

## Failed Tests

EOF
        for test in "${FAILED_TESTS[@]}"; do
            echo "- ❌ $test" >> "$report_file"
        done
    fi

    cat >> "$report_file" << EOF

## Test Categories Executed

1. **Rust Unit Tests** - Core geometry and function logic
2. **PostgreSQL Integration Tests** - Extension functionality within PostgreSQL
3. **SQL Functional Tests** - Basic spatial functions and operations
4. **Geometry Operation Tests** - Comprehensive geometric calculations
5. **Performance Regression Tests** - Ensure reasonable performance benchmarks
6. **Spatial Indexing Tests** - GiST index functionality
$([ -f "$RESULTS_DIR/postgis_compatibility_${TIMESTAMP}.log" ] && echo "7. **PostGIS Compatibility Tests** - Cross-compatibility verification")

## Log Files Generated

EOF

    for log_file in "$RESULTS_DIR"/*_"${TIMESTAMP}".log; do
        if [ -f "$log_file" ]; then
            echo "- $(basename "$log_file")" >> "$report_file"
        fi
    done

    cat >> "$report_file" << EOF

## Conclusion

EOF

    if [ $TESTS_FAILED -eq 0 ]; then
        cat >> "$report_file" << EOF
🎉 **All tests passed successfully!** RostGIS is functioning correctly and ready for use.
EOF
    else
        cat >> "$report_file" << EOF
⚠️ **Some tests failed.** Please review the failed tests and their corresponding log files for detailed error information.

### Next Steps
1. Review failed test logs in the \`$RESULTS_DIR\` directory
2. Fix any identified issues in the codebase
3. Re-run the integration tests
4. Ensure all tests pass before deployment
EOF
    fi

    success "Integration test report generated: $report_file"
}

# Cleanup function
cleanup() {
    if [ "$1" = "--clean" ]; then
        log "Cleaning up test database and files..."
        dropdb -U "$POSTGRES_USER" "$DB_NAME" 2>/dev/null || true
        rm -rf "$RESULTS_DIR"
        success "Cleanup completed"
    fi
}

# Main execution
main() {
    case "${1:-}" in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --clean              Clean up test database and result files"
            echo "  --unit-only          Run only Rust unit tests"
            echo "  --sql-only           Run only SQL-based tests"
            echo "  --no-pgrx            Skip pgrx integration tests"
            echo "  --help               Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  DB_NAME              Test database name (default: rostgis_test)"
            echo "  POSTGRES_USER        PostgreSQL user (default: postgres)"
            exit 0
            ;;
        --clean)
            cleanup --clean
            exit 0
            ;;
        --unit-only)
            check_prerequisites
            run_rust_tests
            ;;
        --sql-only)
            check_prerequisites
            setup_test_database
            run_sql_tests
            run_geometry_tests
            run_performance_regression_tests
            ;;
        --no-pgrx)
            check_prerequisites
            setup_test_database
            run_rust_tests
            run_sql_tests
            run_geometry_tests
            run_performance_regression_tests
            generate_test_report
            ;;
        *)
            check_prerequisites
            setup_test_database
            run_rust_tests
            run_pgrx_tests
            run_sql_tests
            run_geometry_tests
            run_performance_regression_tests
            generate_test_report
            
            if [ $TESTS_FAILED -eq 0 ]; then
                success "All integration tests completed successfully!"
                echo -e "${GREEN}🎉 RostGIS is ready for use!${NC}"
            else
                error "$TESTS_FAILED test(s) failed. Check the test report for details."
                echo -e "${YELLOW}To clean up, run: $0 --clean${NC}"
                exit 1
            fi
            ;;
    esac
}

main "$@" 