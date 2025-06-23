# RostGIS

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?maxAge=3600)](https://github.com/rust-lang/rust)

RostGIS is a high-performance, PostGIS-compatible spatial extension for PostgreSQL written in Rust using the [pgrx](https://github.com/tcdi/pgrx) framework. The name "RostGIS" combines "Rost" (German for "Rust") with "GIS", reflecting both its implementation language and its geospatial focus. It aims to provide 100% data type and function compatibility with PostGIS while potentially offering improved performance through Rust's memory safety and zero-cost abstractions.

## Features

### ✅ Currently Implemented

- **Geometry Data Types**: Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, GeometryCollection
- **Core Construction Functions**:
  - `ST_GeomFromText()` / `ST_GeomFromWKT()` - Create geometry from Well-Known Text
  - `ST_MakePoint()` / `ST_Point()` - Create point geometries
  - `ST_MakePointZ()` - Create 3D points (basic support)
- **Geometry Output Functions**:
  - `ST_AsText()` / `ST_AsWKT()` - Convert geometry to WKT
  - `ST_AsWKB()` - Convert geometry to Well-Known Binary (hex format)
  - `ST_AsGeoJSON()` - Convert geometry to GeoJSON
- **Geometry Property Functions**:
  - `ST_X()`, `ST_Y()`, `ST_Z()` - Extract coordinates
  - `ST_GeometryType()` - Get geometry type
  - `ST_SRID()` - Get spatial reference system ID
  - `ST_SetSRID()` - Set spatial reference system ID
- **Spatial Relationship Functions**:
  - `ST_Equals()` - Test geometric equality
  - `ST_Distance()` - Calculate Euclidean distance
- **Measurement Functions**:
  - `ST_Area()` - Calculate area of polygons
  - `ST_Length()` - Calculate length of linear geometries
  - `ST_Perimeter()` - Calculate perimeter of polygons

### 🚧 Planned Features

- **Additional Construction Functions**: ST_GeomFromWKB, ST_MakeLine, ST_MakePolygon
- **Spatial Analysis**: ST_Intersects, ST_Contains, ST_Within, ST_Touches, ST_Crosses
- **Geometric Operations**: ST_Buffer, ST_Intersection, ST_Union, ST_Difference
- **Spatial Indexing**: GiST index support
- **Coordinate Reference Systems**: Full PROJ integration
- **3D Geometry Support**: Complete Z/M coordinate support

## Installation

### Prerequisites

- PostgreSQL 13-17
- Rust 1.70+
- pgrx (installed via `cargo install --locked cargo-pgrx`)

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/rostgis.git
   cd rostgis
   ```

2. Initialize pgrx (if not already done):
   ```bash
   cargo pgrx init
   ```

3. Build and install the extension:
   ```bash
   cargo pgrx install
   ```

4. In your PostgreSQL database, create the extension:
   ```sql
   CREATE EXTENSION rostgis;
   ```

## Usage

### Basic Examples

```sql
-- Create the extension
CREATE EXTENSION rostgis;

-- Check version
SELECT rostgis_version();

-- Create a point
SELECT ST_MakePoint(1.0, 2.0);

-- Parse WKT
SELECT ST_GeomFromText('POINT(1 2)');

-- Get geometry properties
SELECT ST_X(ST_MakePoint(1.0, 2.0));  -- Returns 1.0
SELECT ST_Y(ST_MakePoint(1.0, 2.0));  -- Returns 2.0

-- Convert to different formats
SELECT ST_AsText(ST_MakePoint(1.0, 2.0));     -- Returns 'POINT(1 2)'
SELECT ST_AsGeoJSON(ST_MakePoint(1.0, 2.0));  -- Returns GeoJSON

-- Spatial operations
SELECT ST_Distance(
    ST_MakePoint(0, 0),
    ST_MakePoint(3, 4)
);  -- Returns 5.0

-- Work with polygons
SELECT ST_Area(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))'));
-- Returns 100.0
```

### PostGIS Compatibility

RostGIS aims for 100% compatibility with PostGIS function signatures and behavior:

```sql
-- These work identically to PostGIS
SELECT ST_GeomFromWKT('LINESTRING(0 0, 1 1, 2 2)');
SELECT ST_SRID(ST_SetSRID(ST_MakePoint(-122.4194, 37.7749), 4326));
SELECT ST_AsText(ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))'));
```

## Performance

RostGIS includes comprehensive benchmarks to ensure optimal performance:

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench -- wkt_parsing
cargo bench -- geometry_operations
```

### Sample Performance Results

*Note: Benchmarks will vary based on your hardware*

```
make_point              time:   [2.5 ns 2.6 ns 2.7 ns]
wkt_parsing/point       time:   [45.2 ns 46.1 ns 47.0 ns]
distance_calculation    time:   [3.1 ns 3.2 ns 3.3 ns]
area_calculation        time:   [8.9 ns 9.2 ns 9.5 ns]
```

## Testing

RostGIS includes comprehensive test suites:

### Unit Tests

```bash
# Run Rust unit tests
cargo test

# Run with output
cargo test -- --nocapture
```

### PostgreSQL Integration Tests

```bash
# Run pgrx tests (requires PostgreSQL)
cargo pgrx test
```

### Property-Based Testing

The project uses [proptest](https://github.com/proptest-rs/proptest) for property-based testing:

```bash
cargo test prop_
```

## Development

### Project Structure

```
rostgis/
├── src/
│   ├── lib.rs           # Main extension entry point
│   ├── geometry.rs      # Geometry type definitions
│   ├── functions.rs     # Spatial function implementations
│   └── utils.rs         # Utility functions
├── benches/
│   └── geometry_benchmarks.rs  # Performance benchmarks
├── sql/                 # SQL test files
└── tests/               # Integration tests
```

### Adding New Functions

1. Add the function signature to `src/lib.rs` with the `#[pg_extern]` attribute
2. Implement the core logic in `src/functions.rs`
3. Add unit tests in the respective module
4. Add integration tests in the `tests` module
5. Add benchmarks if the function is performance-critical

### Code Quality

The project maintains high code quality through:

- **Linting**: `cargo clippy`
- **Formatting**: `cargo fmt`
- **Testing**: Comprehensive unit and integration tests
- **Benchmarking**: Performance regression detection
- **Documentation**: Inline docs and examples

## Comparison with PostGIS

| Feature          | PostGIS | RostGIS | Status             |
|------------------|---------|---------|--------------------|
| Geometry Types   | ✅       | ✅       | Complete           |
| WKT/WKB Support  | ✅       | 🚧      | Partial            |
| Spatial Indexing | ✅       | 🚧      | Planned            |
| 3D Support       | ✅       | 🚧      | Partial            |
| Raster Support   | ✅       | ❌       | Not Planned        |
| Topology         | ✅       | 🚧      | Planned            |
| Performance      | Good    | ⚡       | Potentially Better |
| Memory Safety    | ❌       | ✅       | Rust Advantage     |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas where help is needed:

1. **WKB Parser Implementation**: Full binary WKB support
2. **Spatial Indexing**: GiST index integration
3. **PROJ Integration**: Coordinate reference system support
4. **Performance Optimization**: SIMD operations, parallel processing
5. **Documentation**: Examples, tutorials, API documentation

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [PostGIS](https://postgis.net/) - The original spatial extension that inspired this project
- [pgrx](https://github.com/tcdi/pgrx) - PostgreSQL extension framework for Rust
- [geo](https://github.com/georust/geo) - Rust geospatial primitives and algorithms
- [GeoRust](https://github.com/georust) - Rust geospatial ecosystem

## Links

- [Documentation](https://docs.rs/rostgis)
- [Crates.io](https://crates.io/crates/rostgis)
- [Issues](https://github.com/yourusername/rostgis/issues)
- [Discussions](https://github.com/yourusername/rostgis/discussions) 