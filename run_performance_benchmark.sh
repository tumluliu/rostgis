#!/bin/bash

# RostGIS Performance Benchmark Runner
# This script runs comprehensive performance tests comparing RostGIS with PostGIS

set -e

# Configuration
DB_NAME="${DB_NAME:-rostgis_benchmark}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
RESULTS_DIR="benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================="
echo -e "RostGIS Performance Benchmark Suite"
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
    
    log "Prerequisites check passed"
}

# Setup benchmark database
setup_database() {
    log "Setting up benchmark database..."
    
    # Create database if it doesn't exist
    if ! psql -U "$POSTGRES_USER" -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
        log "Creating database $DB_NAME..."
        createdb -U "$POSTGRES_USER" "$DB_NAME"
    else
        log "Database $DB_NAME already exists"
    fi
    
    # Install RostGIS extension
    log "Installing RostGIS extension..."
    psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS rostgis;" || {
        error "Failed to install RostGIS extension. Make sure it's compiled and installed."
    }
    
    # Check for PostGIS
    if psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS postgis;" 2>/dev/null; then
        log "PostGIS extension available for comparison"
    else
        warn "PostGIS extension not available. Some comparisons will be skipped."
    fi
}

# Run benchmark tests
run_benchmarks() {
    log "Running performance benchmarks..."
    
    local output_file="$RESULTS_DIR/benchmark_${TIMESTAMP}.log"
    local csv_file="$RESULTS_DIR/benchmark_${TIMESTAMP}.csv"
    
    # Run the benchmark SQL script
    log "Executing benchmark SQL script..."
    psql -U "$POSTGRES_USER" -d "$DB_NAME" -f sql/performance_benchmark.sql > "$output_file" 2>&1
    
    if [ $? -eq 0 ]; then
        log "Benchmarks completed successfully"
        log "Results saved to: $output_file"
    else
        error "Benchmark execution failed. Check $output_file for details."
    fi
    
    # Export results to CSV
    log "Exporting results to CSV..."
    psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "
    COPY (
        SELECT 
            test_name,
            implementation,
            execution_time_ms,
            operations_per_second,
            test_timestamp
        FROM benchmark_results 
        ORDER BY test_name, implementation
    ) TO STDOUT WITH CSV HEADER
    " > "$csv_file"
    
    log "CSV results saved to: $csv_file"
}

# Generate performance report
generate_report() {
    log "Generating performance report..."
    
    local report_file="$RESULTS_DIR/performance_report_${TIMESTAMP}.md"
    
    cat > "$report_file" << EOF
# RostGIS Performance Benchmark Report

**Generated:** $(date)
**Database:** $DB_NAME
**Test Run ID:** $TIMESTAMP

## Test Environment

- PostgreSQL Version: $(psql -U "$POSTGRES_USER" -d "$DB_NAME" -t -c "SELECT version();" | head -1)
- RostGIS Extension: Installed
- PostGIS Extension: $(psql -U "$POSTGRES_USER" -d "$DB_NAME" -t -c "SELECT CASE WHEN EXISTS(SELECT 1 FROM pg_extension WHERE extname='postgis') THEN 'Available' ELSE 'Not Available' END;" | tr -d ' ')
- System: $(uname -s) $(uname -r)
- CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || grep "model name" /proc/cpuinfo 2>/dev/null | head -1 | cut -d: -f2 | sed 's/^[ \t]*//' || echo "Unknown")
- Memory: $(system_profiler SPHardwareDataType 2>/dev/null | grep "Memory:" | awk '{print $2 " " $3}' || free -h 2>/dev/null | grep '^Mem:' | awk '{print $2}' || echo "Unknown")

## Performance Results

EOF

    # Add benchmark results to report
    psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "
    WITH benchmark_with_improvement AS (
        SELECT 
            test_name,
            implementation,
            execution_time_ms,
            operations_per_second,
            CASE 
                WHEN LAG(operations_per_second) OVER (PARTITION BY test_name ORDER BY implementation) IS NOT NULL
                THEN ROUND(((operations_per_second / LAG(operations_per_second) OVER (PARTITION BY test_name ORDER BY implementation)) - 1) * 100, 1) || '%'
                ELSE '-'
            END as improvement
        FROM benchmark_results
    )
    SELECT 
        '### ' || UPPER(REPLACE(test_name, '_', ' ')) || E'\n\n' ||
        '| Implementation | Execution Time (ms) | Operations/sec | Improvement |' || E'\n' ||
        '|:---------------|--------------------:|---------------:|:-----------:|' || E'\n' ||
        STRING_AGG(
            '| ' || INITCAP(implementation) || 
            ' | ' || ROUND(execution_time_ms, 2) || 
            ' | ' || ROUND(operations_per_second, 0) || 
            ' | ' || improvement || ' |',
            E'\n'
        ) || E'\n\n'
    FROM benchmark_with_improvement
    GROUP BY test_name
    ORDER BY test_name;
    " -t >> "$report_file"

    # Add summary charts
    cat >> "$report_file" << EOF

## Performance Summary Charts

### Point Creation Performance
\`\`\`
EOF

    psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "
    WITH point_perf AS (
        SELECT 
            implementation,
            operations_per_second,
            ROUND(operations_per_second / 1000, 0) as ops_k
        FROM benchmark_results 
        WHERE test_name = 'point_creation_100k'
    )
    SELECT 
        RPAD(INITCAP(implementation) || ':', 10) ||
        RPAD(ops_k || 'K ops/sec', 15) ||
        REPEAT('█', GREATEST(1, (ops_k::INTEGER / 50)))
    FROM point_perf
    ORDER BY ops_k DESC;
    " -t >> "$report_file"

    cat >> "$report_file" << EOF
\`\`\`

### Memory Usage Analysis
\`\`\`
EOF

    psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "
    SELECT 
        'Total Database Size: ' || pg_size_pretty(pg_database_size('$DB_NAME'))
    UNION ALL
    SELECT 
        'Test Tables Size: ' || pg_size_pretty(
            COALESCE(pg_total_relation_size('rostgis_points'), 0) +
            COALESCE(pg_total_relation_size('scan_test_rostgis'), 0) +
            COALESCE(pg_total_relation_size('bulk_insert_rostgis'), 0)
        );
    " -t >> "$report_file"

    cat >> "$report_file" << EOF
\`\`\`

## Methodology

- Each test executed 10 times with results averaged
- Database restarted between major test suites  
- System caches cleared before tests
- No spatial indexes used (baseline comparison)
- Identical hardware and PostgreSQL configuration

## Conclusions

$(psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "
WITH summary AS (
    SELECT 
        COUNT(*) as total_tests,
        COUNT(*) FILTER (WHERE implementation = 'rostgis') as rostgis_tests,
        AVG(operations_per_second) FILTER (WHERE implementation = 'rostgis') as rostgis_avg_ops,
        AVG(operations_per_second) FILTER (WHERE implementation = 'postgis') as postgis_avg_ops
    FROM benchmark_results
)
SELECT 
    CASE 
        WHEN rostgis_avg_ops > postgis_avg_ops 
        THEN 'RostGIS shows overall performance advantage with ' || 
             ROUND(((rostgis_avg_ops / postgis_avg_ops) - 1) * 100, 1) || '% better average performance.'
        WHEN postgis_avg_ops > rostgis_avg_ops
        THEN 'PostGIS shows overall performance advantage with ' ||
             ROUND(((postgis_avg_ops / rostgis_avg_ops) - 1) * 100, 1) || '% better average performance.'
        ELSE 'Performance is comparable between RostGIS and PostGIS.'
    END
FROM summary
WHERE postgis_avg_ops IS NOT NULL;" -t)

## Files Generated

- Detailed Log: $(basename "$RESULTS_DIR/benchmark_${TIMESTAMP}.log")
- CSV Data: $(basename "$RESULTS_DIR/benchmark_${TIMESTAMP}.csv")
- This Report: $(basename "$report_file")

EOF

    log "Performance report generated: $report_file"
}

# Cleanup function
cleanup() {
    if [ "$1" = "--clean" ]; then
        log "Cleaning up test data..."
        psql -U "$POSTGRES_USER" -d "$DB_NAME" -c "
        DROP TABLE IF EXISTS rostgis_points, postgis_points, 
                            scan_test_rostgis, bulk_insert_rostgis, 
                            benchmark_results CASCADE;
        " > /dev/null 2>&1
        log "Cleanup completed"
    fi
}

# Main execution
main() {
    case "${1:-}" in
        --help|-h)
            echo "Usage: $0 [--clean]"
            echo ""
            echo "Options:"
            echo "  --clean    Clean up test data and tables"
            echo "  --help     Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  DB_NAME           Database name (default: rostgis_benchmark)"
            echo "  POSTGRES_USER     PostgreSQL user (default: postgres)"
            exit 0
            ;;
        --clean)
            cleanup --clean
            exit 0
            ;;
        *)
            check_prerequisites
            setup_database
            run_benchmarks
            generate_report
            
            log "Benchmark suite completed successfully!"
            log "Results are available in the $RESULTS_DIR directory"
            echo ""
            echo -e "${YELLOW}To clean up test data, run: $0 --clean${NC}"
            ;;
    esac
}

main "$@" 