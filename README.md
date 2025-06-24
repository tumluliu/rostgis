# RostGIS

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?maxAge=3600)](https://github.com/rust-lang/rust)

RostGIS is a high-performance, PostGIS-compatible spatial extension for PostgreSQL written in Rust using the [pgrx](https://github.com/tcdi/pgrx) framework. The name "RostGIS" combines "Rost" (German for "Rust") with "GIS", reflecting both its implementation language and its geospatial focus.

## 🚀 Quick Start

```sql
-- Install extension
CREATE EXTENSION rostgis;

-- Create spatial data
CREATE TABLE locations (
    id SERIAL PRIMARY KEY,
    name TEXT,
    location GEOMETRY
);

-- Insert data with spatial indexing
INSERT INTO locations (name, location) VALUES
    ('San Francisco', ST_MakePoint(-122.4194, 37.7749)),
    ('New York', ST_MakePoint(-74.0060, 40.7128));

-- Set up spatial indexing
\i sql/gist_index_setup.sql
CREATE INDEX locations_idx ON locations USING GIST (location gist_geometry_ops_simple);

-- Run spatial queries
SELECT name FROM locations 
WHERE location && ST_MakePoint(-122.4, 37.7);
```

## ✨ Key Features

- **PostGIS Compatibility** - Drop-in replacement for common PostGIS functions
- **High Performance** - 3.39M point operations/sec, 3.92M distance calculations/sec
- **Spatial Indexing** - Full GiST index support for efficient spatial queries
- **Memory Safe** - Rust's safety guarantees eliminate spatial data vulnerabilities
- **Modern Architecture** - Clean, modular design built on pgrx framework

## 📚 Documentation

### 🎯 Getting Started
- **[Installation Guide](docs/user-guide/INSTALLATION.md)** - Complete installation instructions
- **[Getting Started](docs/user-guide/GETTING_STARTED.md)** - Basic usage and examples
- **[Basic Queries Tutorial](docs/tutorials/BASIC_QUERIES.md)** - Step-by-step spatial query tutorial

### 📖 User Guides
- **[Spatial Indexing](docs/user-guide/SPATIAL_INDEXING.md)** - Creating and using spatial indexes
- **[Performance Benchmarking](docs/user-guide/PERFORMANCE_BENCHMARKING.md)** - Performance testing and optimization
- **[Migration from PostGIS](docs/user-guide/MIGRATION.md)** - Migration guide for PostGIS users

### 👨‍💻 Developer Resources
- **[API Reference](docs/api-reference/SPATIAL_FUNCTIONS.md)** - Complete function documentation
- **[Architecture Overview](docs/developer-guide/ARCHITECTURE.md)** - System design and implementation
- **[Contributing Guide](docs/developer-guide/CONTRIBUTING.md)** - How to contribute to development

### 📋 Complete Documentation
See **[docs/README.md](docs/README.md)** for the full documentation index.

## 🏗️ Current Implementation Status

### ✅ Fully Implemented
- **Core Geometry Types**: Point, LineString, Polygon, Multi*, GeometryCollection
- **PostGIS Functions**: 20+ compatible functions including ST_MakePoint, ST_Distance, ST_Intersects
- **Spatial Indexing**: Complete GiST implementation with R-Tree spatial partitioning
- **Input/Output**: WKT, WKB, and GeoJSON support
- **Performance**: Benchmark suite with real performance metrics

### 🚧 In Development
- Advanced geometric operations (ST_Buffer, ST_Union)
- Coordinate reference system transformations
- Enhanced 3D geometry support

## 🚀 Performance Highlights

Based on real benchmark results:

| Operation             | Performance        | Use Case                            |
|-----------------------|--------------------|-------------------------------------|
| Point Creation        | 3.39M ops/sec      | GPS tracking, real-time location    |
| Distance Calculations | 3.92M ops/sec      | Proximity queries, spatial analysis |
| WKT Parsing           | 0.87-2.74M ops/sec | Data loading, format conversion     |
| Spatial Queries       | Index-accelerated  | Large dataset spatial analysis      |

**Memory Efficiency**: 16MB total for comprehensive test datasets

See [Performance Benchmarking Guide](docs/user-guide/PERFORMANCE_BENCHMARKING.md) for detailed results.

## 📦 Installation

### Quick Installation
```bash
# Install pgrx
cargo install pgrx --version="=0.11.2"
cargo pgrx init

# Clone and install RostGIS
git clone https://github.com/yourusername/rostgis.git
cd rostgis
cargo pgrx install
```

### Enable in PostgreSQL
```sql
CREATE EXTENSION rostgis;
SELECT rostgis_version();
```

For detailed installation instructions, see [Installation Guide](docs/user-guide/INSTALLATION.md).

## 🔧 Function Compatibility

| Function          | PostGIS | RostGIS | Notes                       |
|-------------------|---------|---------|-----------------------------|
| ST_MakePoint      | ✅       | ✅       | Identical                   |
| ST_GeomFromText   | ✅       | ✅       | Full WKT support            |
| ST_AsText         | ✅       | ✅       | Same output format          |
| ST_Distance       | ✅       | ✅       | High performance            |
| ST_Intersects     | ✅       | ✅       | Index accelerated           |
| ST_Contains       | ✅       | ✅       | Index accelerated           |
| Spatial Operators | ✅       | ✅       | Full `&&`, `<<`, `>>`, etc. |

See [Migration Guide](docs/user-guide/MIGRATION.md) for complete compatibility information.

## 🎯 Use Cases

### GPS Tracking & IoT
- **High-throughput** point insertion (970K+ points/sec)
- **Real-time** proximity queries
- **Memory efficient** storage for millions of locations

### Geospatial Analytics
- **Fast distance** calculations (3.92M ops/sec)
- **Index-accelerated** spatial joins
- **Efficient** geometric property analysis

### Web Mapping Applications
- **GeoJSON export** at 3.86M points/sec
- **Spatial indexing** for interactive maps
- **PostGIS compatibility** for easy migration

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](docs/developer-guide/CONTRIBUTING.md) for:
- Development environment setup
- Code style guidelines  
- Testing requirements
- Pull request process

### High-Priority Areas
- PostGIS function implementations
- Performance optimizations
- Documentation improvements
- Test coverage expansion

## 📜 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🔗 Links

- **Documentation**: [docs/README.md](docs/README.md)
- **GitHub Issues**: Report bugs and request features
- **Performance Benchmarks**: [Benchmarking Guide](docs/user-guide/PERFORMANCE_BENCHMARKING.md)
- **PostGIS Migration**: [Migration Guide](docs/user-guide/MIGRATION.md)

---

*RostGIS: Bringing Rust's performance and safety to PostgreSQL spatial data processing.* 