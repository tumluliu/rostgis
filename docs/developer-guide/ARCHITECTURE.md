# RostGIS Architecture Overview

This document provides a comprehensive overview of RostGIS's architecture, design decisions, and implementation details.

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    PostgreSQL Server                        │
├─────────────────────────────────────────────────────────────┤
│                    RostGIS Extension                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Spatial   │ │  Geometry   │ │    Spatial Indexing     │ │
│  │ Functions   │ │    Types    │ │     (GiST Support)      │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Parser    │ │  Operators  │ │      Utilities          │ │
│  │  (WKT/WKB)  │ │ (Spatial)   │ │   (Memory Management)   │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                     pgrx Framework                          │
├─────────────────────────────────────────────────────────────┤
│                      Rust Runtime                           │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Geometry Types (`src/geometry.rs`)
- **Purpose**: Core spatial data types and operations
- **Key Types**:
  - `Point` - 2D/3D point coordinates
  - `LineString` - Connected line segments  
  - `Polygon` - Closed polygonal areas
  - `MultiPoint`, `MultiLineString`, `MultiPolygon` - Collections
  - `GeometryCollection` - Mixed geometry collections

#### 2. Spatial Functions (`src/functions.rs`)
- **Purpose**: PostGIS-compatible spatial operations
- **Categories**:
  - Construction: `ST_MakePoint`, `ST_GeomFromText`
  - Output: `ST_AsText`, `ST_AsGeoJSON`, `ST_AsWKB`
  - Properties: `ST_X`, `ST_Y`, `ST_Area`, `ST_Length`
  - Relationships: `ST_Distance`, `ST_Intersects`, `ST_Contains`

#### 3. Spatial Indexing (`src/spatial_index.rs`)
- **Purpose**: GiST index support for efficient spatial queries
- **Components**:
  - `BBox` type for bounding box operations
  - GiST support functions (compress, decompress, union)
  - Spatial operators (`&&`, `<<`, `>>`, `~`, `@`)

#### 4. Main Library (`src/lib.rs`)
- **Purpose**: Extension entry point and PostgreSQL integration
- **Responsibilities**:
  - Function exports with `#[pg_extern]`
  - Operator definitions with `#[pg_operator]`
  - Extension metadata and initialization

#### 5. Utilities (`src/utils.rs`)
- **Purpose**: Helper functions and shared utilities
- **Functions**: Error handling, memory management, type conversions

## Design Principles

### 1. PostGIS Compatibility
**Goal**: Drop-in replacement for PostGIS functions

**Implementation**:
- Identical function signatures and return types
- Same WKT/WKB formats
- Compatible spatial operators and indexing

**Example**:
```rust
#[pg_extern]
fn st_makepoint(x: f64, y: f64) -> Geometry {
    make_point(x, y)  // Delegates to functions.rs
}
```

### 2. Performance First
**Goal**: Leverage Rust's performance characteristics

**Strategies**:
- Zero-cost abstractions where possible
- Efficient memory management
- SIMD-friendly data structures
- Minimal allocations in hot paths

**Example**:
```rust
// Efficient bounding box operations
impl Geometry {
    pub fn bbox_overlaps(&self, other: &Geometry) -> bool {
        let (min_x1, min_y1, max_x1, max_y1) = self.bounding_box();
        let (min_x2, min_y2, max_x2, max_y2) = other.bounding_box();
        
        max_x1 >= min_x2 && min_x1 <= max_x2 && 
        max_y1 >= min_y2 && min_y1 <= max_y2
    }
}
```

### 3. Memory Safety
**Goal**: Eliminate spatial-related memory vulnerabilities

**Benefits**:
- No buffer overflows in geometry parsing
- Safe handling of large datasets
- Predictable memory usage patterns

**Implementation**:
- Rust's ownership system prevents use-after-free
- Bounds checking on coordinate access
- Safe FFI with PostgreSQL

### 4. Modularity
**Goal**: Clean separation of concerns for maintainability

**Structure**:
```
src/
├── lib.rs          # PostgreSQL interface
├── geometry.rs     # Core geometry types
├── functions.rs    # Spatial algorithms
├── spatial_index.rs # Indexing support
└── utils.rs        # Shared utilities
```

## Data Types and Representation

### Geometry Type Hierarchy

```rust
#[derive(PostgresType, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
}
```

### Internal Representation

#### Point Storage
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,  // Optional 3D coordinate
    pub srid: i32,       // Spatial Reference System ID
}
```

#### LineString Storage
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct LineString {
    pub points: Vec<Point>,
    pub srid: i32,
}
```

#### Memory Layout Optimization
- Contiguous storage for coordinates
- Minimal header overhead
- Cache-friendly data access patterns

### Bounding Box Representation

```rust
#[derive(PostgresType, Debug, Clone, PartialEq)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
```

## PostgreSQL Integration

### pgrx Framework Integration

RostGIS leverages the pgrx framework for PostgreSQL integration:

```rust
use pgrx::prelude::*;

::pgrx::pg_module_magic!();  // Required for extension loading

#[pg_extern]                 // Exports function to PostgreSQL
fn st_makepoint(x: f64, y: f64) -> Geometry {
    make_point(x, y)
}
```

### Type System Integration

#### Custom PostgreSQL Types
```rust
#[derive(PostgresType)]
#[inoutfuncs]  // Automatic input/output functions
pub enum Geometry {
    // ... variants
}
```

#### Operator Registration
```rust
#[pg_operator(immutable, parallel_safe)]
#[opname(&&)]  // Spatial overlap operator
fn geometry_overlap(left: Geometry, right: Geometry) -> bool {
    left.bbox_overlaps(&right)
}
```

### GiST Index Integration

#### Support Functions
```rust
#[pg_extern]
fn geometry_gist_compress(geom: Geometry) -> BBox {
    let (min_x, min_y, max_x, max_y) = geom.bounding_box();
    BBox { min_x, min_y, max_x, max_y }
}

#[pg_extern] 
fn bbox_union(a: BBox, b: BBox) -> BBox {
    BBox {
        min_x: a.min_x.min(b.min_x),
        min_y: a.min_y.min(b.min_y),
        max_x: a.max_x.max(b.max_x),
        max_y: a.max_y.max(b.max_y),
    }
}
```

#### Operator Class Definition
```sql
CREATE OPERATOR CLASS gist_geometry_ops_simple
DEFAULT FOR TYPE geometry USING gist AS
    OPERATOR 1 &&,
    OPERATOR 2 <<,
    -- ... other operators
    FUNCTION 1 geometry_gist_compress(geometry),
    FUNCTION 2 bbox_union(bbox, bbox);
```

## Parsing and Serialization

### WKT (Well-Known Text) Parser

#### Design Approach
- Recursive descent parser
- Error recovery and meaningful error messages
- Support for all OGC geometry types

#### Implementation Structure
```rust
pub fn parse_wkt(wkt: &str) -> Result<Geometry, WktError> {
    let mut parser = WktParser::new(wkt);
    parser.parse_geometry()
}

impl WktParser {
    fn parse_point(&mut self) -> Result<Point, WktError> {
        self.expect_token("POINT")?;
        self.expect_token("(")?;
        let x = self.parse_number()?;
        let y = self.parse_number()?;
        self.expect_token(")")?;
        Ok(Point::new(x, y))
    }
}
```

### WKB (Well-Known Binary) Support

#### Binary Format Handling
- Endianness detection and handling
- Efficient binary parsing
- Compact storage representation

### GeoJSON Integration

#### JSON Serialization
```rust
impl Geometry {
    pub fn to_geojson(&self) -> String {
        match self {
            Geometry::Point(p) => format!(
                r#"{{"type":"Point","coordinates":[{},{}]}}"#,
                p.x, p.y
            ),
            // ... other types
        }
    }
}
```

## Spatial Indexing Architecture

### GiST Integration Strategy

#### Bounding Box Approach
- All geometries compressed to bounding boxes for indexing
- R-Tree like spatial partitioning
- Balance between index size and query performance

#### Index Operations
```rust
// Key index operations
fn compress(geom: Geometry) -> BBox;      // Convert geometry to index key
fn union(a: BBox, b: BBox) -> BBox;       // Combine bounding boxes
fn penalty(a: BBox, b: BBox) -> f64;      // Cost of adding b to a
fn same(a: BBox, b: BBox) -> bool;        // Test equality
```

### Query Processing

#### Index-Accelerated Queries
1. **Spatial Operator**: `geom && ST_MakePoint(x, y)`
2. **Index Scan**: PostgreSQL uses GiST index to find candidates
3. **Filter Phase**: Apply exact geometric predicate if needed

#### Query Optimization
- Bounding box pre-filtering
- Early termination for obvious cases
- Minimal exact geometry computation

## Performance Considerations

### Memory Management

#### Allocation Strategy
- Stack allocation for small geometries
- Pool allocation for temporary objects
- Minimal heap allocation in hot paths

#### Cache Efficiency
- Contiguous coordinate storage
- Predictable memory access patterns
- SIMD-friendly data layout

### Computational Optimization

#### Fast Path Operations
```rust
// Optimized distance calculation
pub fn point_distance_squared(p1: &Point, p2: &Point) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    dx * dx + dy * dy  // Avoid sqrt when possible
}
```

#### Algorithmic Choices
- Use bounding box tests before expensive operations
- Implement early exit conditions
- Choose algorithms based on geometry complexity

## Error Handling Strategy

### Error Types
```rust
#[derive(Debug)]
pub enum GeometryError {
    ParseError(String),
    InvalidGeometry(String),
    UnsupportedOperation(String),
    InternalError(String),
}
```

### Error Propagation
- Use Result types for fallible operations
- Convert to PostgreSQL errors at boundary
- Provide meaningful error messages to users

### Recovery Strategies
- Graceful degradation when possible
- Clear error reporting for debugging
- Transaction safety preservation

## Testing Architecture

### Test Categories

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_point_creation() {
        let p = Point::new(1.0, 2.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
    }
}
```

#### Integration Tests
```rust
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    #[pg_test]
    fn test_st_makepoint() {
        let point = crate::st_makepoint(1.0, 2.0);
        assert_eq!(crate::st_x(point.clone()).unwrap(), 1.0);
    }
}
```

#### Performance Tests
```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_point_creation(c: &mut Criterion) {
        c.bench_function("point creation", |b| {
            b.iter(|| Point::new(black_box(1.0), black_box(2.0)))
        });
    }
}
```

## Future Architecture Considerations

### Scalability Improvements

#### Parallel Processing
- SIMD acceleration for bulk operations
- Parallel query processing for large datasets
- Multi-threaded spatial operations

#### Advanced Indexing
- R*-Tree variants for better performance
- Spatial partitioning strategies
- Adaptive index structures

### Feature Extensions

#### 3D Geometry Support
- Z-coordinate handling throughout
- 3D spatial operations
- Volume calculations

#### Coordinate Reference Systems
- PROJ integration for transformations
- CRS metadata handling
- Projection performance optimization

#### Advanced Spatial Analysis
- Topology operations
- Network analysis
- Raster data integration

---

*This architecture document is maintained alongside code changes and represents the current system design.* 