# Testing Guide for RostGIS

Comprehensive guide to testing RostGIS functionality, performance, and compatibility.

## Testing Philosophy

RostGIS follows a comprehensive testing strategy:
- **Unit Tests** - Test individual functions in isolation
- **Integration Tests** - Test PostgreSQL integration
- **Performance Tests** - Benchmark performance characteristics
- **Compatibility Tests** - Verify PostGIS compatibility
- **Property-Based Tests** - Validate geometric properties

## Test Categories

### 1. Unit Tests (Rust)

Test individual Rust functions without PostgreSQL dependency.

#### Running Unit Tests
```bash
# Run all unit tests
cargo test

# Run specific test module
cargo test geometry

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

#### Writing Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(1.0, 2.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, None);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        let distance = point_distance(&p1, &p2);
        assert!((distance - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "Invalid WKT")]
    fn test_invalid_wkt_parsing() {
        parse_wkt("INVALID WKT STRING").unwrap();
    }
}
```

#### Test Organization
```rust
// Test utilities
mod test_utils {
    use super::*;
    
    pub fn create_test_point() -> Point {
        Point::new(1.0, 2.0)
    }
    
    pub fn assert_points_equal(p1: &Point, p2: &Point) {
        assert!((p1.x - p2.x).abs() < f64::EPSILON);
        assert!((p1.y - p2.y).abs() < f64::EPSILON);
    }
}

// Geometry tests
#[cfg(test)]
mod geometry_tests {
    use super::test_utils::*;
    
    #[test]
    fn test_point_equality() {
        let p1 = create_test_point();
        let p2 = create_test_point();
        assert_points_equal(&p1, &p2);
    }
}
```

### 2. Integration Tests (PostgreSQL)

Test RostGIS functions within PostgreSQL using pgrx testing framework.

#### Running Integration Tests
```bash
# Run all PostgreSQL integration tests
cargo pgrx test

# Run with specific PostgreSQL version
cargo pgrx test --pg-version 15

# Run specific test
cargo pgrx test -- test_st_makepoint

# Verbose output
cargo pgrx test -- --nocapture
```

#### Writing Integration Tests
```rust
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::*;
    use pgrx::prelude::*;

    #[pg_test]
    fn test_st_makepoint_integration() {
        let point = crate::st_makepoint(1.0, 2.0);
        assert_eq!(crate::st_x(point.clone()).unwrap(), 1.0);
        assert_eq!(crate::st_y(point).unwrap(), 2.0);
    }

    #[pg_test]
    fn test_spatial_operators() {
        let point1 = crate::st_makepoint(0.0, 0.0);
        let point2 = crate::st_makepoint(1.0, 1.0);
        
        // Test overlap operator
        assert!(crate::geometry_overlap(point1.clone(), point1.clone()));
        
        // Test spatial relationships
        assert!(crate::st_intersects(point1.clone(), point1.clone()));
        assert!(crate::st_dwithin(point1, point2, 2.0));
    }

    #[pg_test]
    fn test_spatial_indexing() {
        // Create test table
        Spi::run("CREATE TABLE test_points (id INT, geom GEOMETRY);").unwrap();
        
        // Insert test data
        Spi::run("INSERT INTO test_points VALUES (1, ST_MakePoint(1, 2));").unwrap();
        
        // Create spatial index
        Spi::run("CREATE INDEX test_idx ON test_points USING GIST (geom gist_geometry_ops_simple);").unwrap();
        
        // Test spatial query
        let result = Spi::get_one::<i32>("SELECT COUNT(*) FROM test_points WHERE geom && ST_MakePoint(1, 2);").unwrap();
        assert_eq!(result, Some(1));
        
        // Cleanup
        Spi::run("DROP TABLE test_points;").unwrap();
    }

    #[pg_test]
    fn test_wkt_round_trip() {
        let original_wkt = "POINT(1.5 2.5)";
        let geom = crate::st_geomfromtext(original_wkt).unwrap();
        let result_wkt = crate::st_astext(geom);
        assert_eq!(result_wkt, "POINT(1.5 2.5)");
    }
}
```

#### Testing SQL Queries
```rust
#[pg_test]
fn test_complex_spatial_query() {
    // Setup test data
    Spi::run("CREATE TABLE cities (id INT, name TEXT, location GEOMETRY);").unwrap();
    Spi::run("INSERT INTO cities VALUES (1, 'SF', ST_MakePoint(-122.4, 37.7));").unwrap();
    Spi::run("INSERT INTO cities VALUES (2, 'LA', ST_MakePoint(-118.2, 34.0));").unwrap();
    
    // Test spatial query
    let result = Spi::get_one::<String>(
        "SELECT name FROM cities WHERE ST_DWithin(location, ST_MakePoint(-122.0, 37.0), 1.0);"
    ).unwrap();
    assert_eq!(result, Some("SF".to_string()));
    
    // Cleanup
    Spi::run("DROP TABLE cities;").unwrap();
}
```

### 3. Performance Tests (Benchmarks)

Measure and track performance characteristics.

#### Running Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- geometry_operations

# Run with baseline comparison
cargo bench -- --save-baseline current
cargo bench -- --baseline current

# Generate benchmark report
cargo bench -- --output-format html
```

#### Writing Benchmarks
```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use super::*;

    fn bench_point_creation(c: &mut Criterion) {
        c.bench_function("point creation", |b| {
            b.iter(|| Point::new(black_box(1.0), black_box(2.0)))
        });
    }

    fn bench_point_distance(c: &mut Criterion) {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        
        c.bench_function("point distance", |b| {
            b.iter(|| point_distance(black_box(&p1), black_box(&p2)))
        });
    }

    fn bench_wkt_parsing(c: &mut Criterion) {
        let wkt_samples = [
            "POINT(1 2)",
            "LINESTRING(0 0, 1 1, 2 2)",
            "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))",
        ];
        
        c.bench_function("wkt parsing", |b| {
            b.iter(|| {
                for wkt in &wkt_samples {
                    parse_wkt(black_box(wkt)).unwrap();
                }
            })
        });
    }

    fn bench_spatial_operations(c: &mut Criterion) {
        let geom1 = Geometry::Point(Point::new(0.0, 0.0));
        let geom2 = Geometry::Point(Point::new(1.0, 1.0));
        
        c.bench_function("bbox overlap", |b| {
            b.iter(|| black_box(&geom1).bbox_overlaps(black_box(&geom2)))
        });
    }

    criterion_group!(benches, 
        bench_point_creation, 
        bench_point_distance, 
        bench_wkt_parsing,
        bench_spatial_operations
    );
    criterion_main!(benches);
}
```

#### PostgreSQL Performance Tests
```bash
# Run complete performance benchmark suite
./run_performance_benchmark.sh

# Custom benchmark parameters
DB_NAME=perf_test SCALE_FACTOR=1000 ./run_performance_benchmark.sh

# Benchmark specific operations
psql -d test_db -f sql/performance_benchmark.sql
```

### 4. SQL Test Scripts

Test PostgreSQL functionality through SQL scripts.

#### Basic Function Tests
```sql
-- test_basic_functions.sql
\echo 'Testing basic spatial functions...'

-- Test point creation
SELECT ST_AsText(ST_MakePoint(1, 2)) = 'POINT(1 2)' AS point_creation_test;

-- Test distance calculation
SELECT abs(ST_Distance(ST_MakePoint(0, 0), ST_MakePoint(3, 4)) - 5.0) < 0.0001 AS distance_test;

-- Test spatial operators
SELECT (ST_MakePoint(1, 1) && ST_MakePoint(1, 1)) AS overlap_test;

\echo 'Basic function tests completed.'
```

#### Spatial Indexing Tests
```sql
-- test_spatial_indexing.sql
\echo 'Testing spatial indexing...'

-- Create test table
CREATE TABLE IF NOT EXISTS test_spatial (
    id SERIAL PRIMARY KEY,
    geom GEOMETRY
);

-- Insert test data
INSERT INTO test_spatial (geom) 
SELECT ST_MakePoint(random() * 100, random() * 100) 
FROM generate_series(1, 1000);

-- Create spatial index
CREATE INDEX test_spatial_idx ON test_spatial 
USING GIST (geom gist_geometry_ops_simple);

-- Test index usage
EXPLAIN (ANALYZE, BUFFERS) 
SELECT COUNT(*) FROM test_spatial 
WHERE geom && ST_MakePoint(50, 50);

-- Cleanup
DROP TABLE test_spatial;

\echo 'Spatial indexing tests completed.'
```

#### PostGIS Compatibility Tests
```sql
-- compare_with_postgis.sql
\echo 'Testing PostGIS compatibility...'

-- Create comparison functions (if PostGIS available)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis') THEN
        -- Compare results between PostGIS and RostGIS
        RAISE NOTICE 'PostGIS detected, running compatibility tests...';
        
        -- Test point creation
        PERFORM (rostgis.ST_AsText(rostgis.ST_MakePoint(1, 2)) = 
                 postgis.ST_AsText(postgis.ST_MakePoint(1, 2)));
        
        -- Test distance calculation
        PERFORM (abs(rostgis.ST_Distance(rostgis.ST_MakePoint(0, 0), rostgis.ST_MakePoint(3, 4)) -
                     postgis.ST_Distance(postgis.ST_MakePoint(0, 0), postgis.ST_MakePoint(3, 4))) < 0.0001);
    ELSE
        RAISE NOTICE 'PostGIS not available, skipping compatibility tests.';
    END IF;
END $$;

\echo 'PostGIS compatibility tests completed.'
```

### 5. Property-Based Testing

Test geometric properties and invariants.

#### Property Test Examples
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_point_roundtrip_wkt(x in -180.0..180.0, y in -90.0..90.0) {
            let point = Point::new(x, y);
            let wkt = point_to_wkt(&point);
            let parsed = parse_wkt(&wkt).unwrap();
            
            if let Geometry::Point(parsed_point) = parsed {
                prop_assert!((parsed_point.x - x).abs() < f64::EPSILON);
                prop_assert!((parsed_point.y - y).abs() < f64::EPSILON);
            } else {
                prop_assert!(false, "Expected Point geometry");
            }
        }

        #[test]
        fn test_distance_symmetry(
            x1 in -100.0..100.0, y1 in -100.0..100.0,
            x2 in -100.0..100.0, y2 in -100.0..100.0
        ) {
            let p1 = Point::new(x1, y1);
            let p2 = Point::new(x2, y2);
            
            let d1 = point_distance(&p1, &p2);
            let d2 = point_distance(&p2, &p1);
            
            prop_assert!((d1 - d2).abs() < f64::EPSILON);
        }

        #[test]
        fn test_distance_triangle_inequality(
            x1 in -50.0..50.0, y1 in -50.0..50.0,
            x2 in -50.0..50.0, y2 in -50.0..50.0,
            x3 in -50.0..50.0, y3 in -50.0..50.0
        ) {
            let p1 = Point::new(x1, y1);
            let p2 = Point::new(x2, y2);
            let p3 = Point::new(x3, y3);
            
            let d12 = point_distance(&p1, &p2);
            let d23 = point_distance(&p2, &p3);
            let d13 = point_distance(&p1, &p3);
            
            // Triangle inequality: d(p1,p3) <= d(p1,p2) + d(p2,p3)
            prop_assert!(d13 <= d12 + d23 + f64::EPSILON);
        }
    }
}
```

## Test Data and Fixtures

### Creating Test Datasets
```rust
// Test data generation
pub mod test_data {
    use super::*;
    
    pub fn create_test_points(count: usize) -> Vec<Point> {
        (0..count)
            .map(|i| Point::new(i as f64, (i * 2) as f64))
            .collect()
    }
    
    pub fn create_random_points(count: usize, seed: u64) -> Vec<Point> {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;
        
        let mut rng = StdRng::seed_from_u64(seed);
        (0..count)
            .map(|_| Point::new(
                rng.gen_range(-180.0..180.0),
                rng.gen_range(-90.0..90.0)
            ))
            .collect()
    }
    
    pub fn create_test_linestring() -> LineString {
        LineString {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(1.0, 1.0),
                Point::new(2.0, 0.0),
            ],
            srid: 0,
        }
    }
}
```

### SQL Test Data
```sql
-- Create test data function
CREATE OR REPLACE FUNCTION create_test_data(scale_factor INT DEFAULT 1000)
RETURNS VOID AS $$
BEGIN
    -- Create test tables
    CREATE TABLE IF NOT EXISTS test_points (
        id SERIAL PRIMARY KEY,
        name TEXT,
        location GEOMETRY
    );
    
    -- Insert test points
    INSERT INTO test_points (name, location)
    SELECT 
        'Point_' || i,
        ST_MakePoint(
            random() * 360 - 180,  -- Longitude: -180 to 180
            random() * 180 - 90    -- Latitude: -90 to 90
        )
    FROM generate_series(1, scale_factor) i;
    
    -- Create spatial index
    CREATE INDEX IF NOT EXISTS test_points_location_idx 
    ON test_points USING GIST (location gist_geometry_ops_simple);
    
    RAISE NOTICE 'Created % test points with spatial index', scale_factor;
END;
$$ LANGUAGE plpgsql;
```

## Test Automation

### Continuous Integration
```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        pg-version: [13, 14, 15, 16, 17]
        
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        
    - name: Install PostgreSQL ${{ matrix.pg-version }}
      run: |
        sudo apt update
        sudo apt install postgresql-${{ matrix.pg-version }} postgresql-server-dev-${{ matrix.pg-version }}
        
    - name: Install pgrx
      run: cargo install pgrx --version="=0.11.2"
      
    - name: Initialize pgrx
      run: cargo pgrx init --pg${{ matrix.pg-version }} $(pg_config --bindir)/pg_config
      
    - name: Run Rust tests
      run: cargo test
      
    - name: Run PostgreSQL tests
      run: cargo pgrx test --pg-version ${{ matrix.pg-version }}
      
    - name: Run SQL tests
      run: |
        cargo pgrx install --pg-version ${{ matrix.pg-version }}
        psql -d postgres -c "CREATE EXTENSION rostgis;"
        psql -d postgres -f sql/test_basic_functions.sql
        
    - name: Run benchmarks
      run: cargo bench -- --output-format json > benchmark_results.json
      
    - name: Check code formatting
      run: cargo fmt -- --check
      
    - name: Run clippy
      run: cargo clippy -- -D warnings
```

### Pre-commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running pre-commit tests..."

# Format check
if ! cargo fmt -- --check; then
    echo "Code formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Lint check
if ! cargo clippy -- -D warnings; then
    echo "Lint check failed. Fix clippy warnings."
    exit 1
fi

# Unit tests
if ! cargo test; then
    echo "Unit tests failed."
    exit 1
fi

# Integration tests (optional - can be slow)
# if ! cargo pgrx test; then
#     echo "Integration tests failed."
#     exit 1
# fi

echo "All pre-commit tests passed!"
```

## Test Coverage

### Measuring Coverage
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html --output-dir coverage/

# View coverage report
open coverage/tarpaulin-report.html

# Generate coverage for CI
cargo tarpaulin --out Xml
```

### Coverage Goals
- **Unit Tests**: >90% line coverage
- **Integration Tests**: All public functions tested
- **Edge Cases**: Error conditions and boundary values
- **Performance**: All critical paths benchmarked

## Testing Best Practices

### Test Organization
```rust
// Organize tests by module
mod geometry_tests {
    mod point_tests { /* point-specific tests */ }
    mod linestring_tests { /* linestring-specific tests */ }
    mod polygon_tests { /* polygon-specific tests */ }
}

mod function_tests {
    mod creation_tests { /* ST_MakePoint, ST_GeomFromText, etc. */ }
    mod property_tests { /* ST_X, ST_Y, ST_Area, etc. */ }
    mod relationship_tests { /* ST_Intersects, ST_Contains, etc. */ }
}
```

### Test Naming Conventions
```rust
#[test]
fn test_[function_name]_[scenario]_[expected_result]() {
    // Example: test_point_distance_zero_distance_returns_zero()
}

#[pg_test]
fn test_[sql_function]_[scenario]() {
    // Example: test_st_makepoint_basic_usage()
}
```

### Assertion Patterns
```rust
// Use appropriate assertions
assert_eq!(actual, expected);  // Exact equality
assert!((actual - expected).abs() < f64::EPSILON);  // Float comparison
assert!(condition, "Custom error message");  // Boolean with message

// Test error conditions
assert!(result.is_err());
assert_eq!(result.unwrap_err().to_string(), "Expected error message");
```

## Debugging Tests

### Test Debugging
```bash
# Run single test with output
cargo test test_point_creation -- --nocapture

# Debug integration test
cargo pgrx test -- test_st_makepoint --nocapture

# Use debugger with tests
rust-gdb target/debug/deps/rostgis-*
(gdb) set args test_point_creation
(gdb) run
```

### PostgreSQL Test Debugging
```rust
#[pg_test]
fn debug_spatial_query() {
    // Enable logging
    log::info!("Starting spatial query test");
    
    // Use Spi::run with debugging
    let result = Spi::run("SELECT ST_AsText(ST_MakePoint(1, 2));");
    log::info!("Query result: {:?}", result);
    
    // Manual verification
    assert!(result.is_ok());
}
```

---

*This testing guide ensures comprehensive coverage of RostGIS functionality. Regular testing maintains code quality and prevents regressions.* 