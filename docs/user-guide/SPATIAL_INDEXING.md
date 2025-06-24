# RostGIS Spatial Indexing Guide

This guide explains how to use spatial indexing in RostGIS for efficient spatial queries.

## Overview

RostGIS provides spatial indexing capabilities through PostgreSQL's GiST (Generalized Search Tree) indexes. This allows for fast spatial queries on large datasets by using bounding box operations.

## Current Status

**🟡 Beta Implementation**: The current spatial indexing implementation provides basic functionality with simplified GiST support functions. This is sufficient for most spatial queries but may have performance limitations compared to full PostGIS implementations.

### What Works
- ✅ Basic spatial operators (`&&`, `<<`, `>>`, `~`, `@`, etc.)
- ✅ GiST index creation and usage
- ✅ Bounding box operations
- ✅ Query acceleration for spatial predicates
- ✅ ST_DWithin and spatial relationship functions

### What's In Development
- 🔄 Full GiST support functions (consistent, picksplit)
- 🔄 Advanced spatial index optimization
- 🔄 Index-only scans
- 🔄 Full PostGIS compatibility

## Quick Start

### 1. Install RostGIS
```sql
CREATE EXTENSION rostgis;
```

### 2. Set Up Spatial Indexing
```bash
# Run the spatial indexing setup script
psql your_database -f sql/gist_index_setup.sql
```

### 3. Create a Table with Geometry Column
```sql
CREATE TABLE locations (
    id SERIAL PRIMARY KEY,
    name TEXT,
    geom GEOMETRY
);
```

### 4. Insert Spatial Data
```sql
INSERT INTO locations (name, geom) VALUES 
    ('San Francisco', ST_MakePoint(-122.4194, 37.7749)),
    ('New York', ST_MakePoint(-74.0060, 40.7128)),
    ('London', ST_MakePoint(-0.1276, 51.5074)),
    ('Tokyo', ST_MakePoint(139.6503, 35.6762));
```

### 5. Create Spatial Index
```sql
CREATE INDEX locations_geom_idx ON locations 
USING GIST (geom gist_geometry_ops_simple);
```

### 6. Run Spatial Queries
```sql
-- Find points near San Francisco
SELECT name, ST_AsText(geom)
FROM locations 
WHERE geom && ST_MakePoint(-122.4, 37.7);

-- Find points within 1000km of San Francisco
SELECT name, ST_Distance(geom, ST_MakePoint(-122.4194, 37.7749)) AS distance_km
FROM locations 
WHERE ST_DWithin(geom, ST_MakePoint(-122.4194, 37.7749), 1000000)
ORDER BY distance_km;
```

## Spatial Operators

RostGIS supports the following spatial operators for indexing:

| Operator | Description                 | Example                |
|----------|-----------------------------|------------------------|
| `&&`     | Bounding boxes overlap      | `geom && other_geom`   |
| `<<`     | Strictly left of            | `geom << other_geom`   |
| `>>`     | Strictly right of           | `geom >> other_geom`   |
| `&<`     | Does not extend to right of | `geom &< other_geom`   |
| `&>`     | Does not extend to left of  | `geom &> other_geom`   |
| `<<\|`   | Strictly below              | `geom <<\| other_geom` |
| `\|>>`   | Strictly above              | `geom \|>> other_geom` |
| `&<\|`   | Does not extend above       | `geom &<\| other_geom` |
| `\|&>`   | Does not extend below       | `geom \|&> other_geom` |
| `~`      | Contains                    | `geom ~ other_geom`    |
| `@`      | Contained by                | `geom @ other_geom`    |
| `~=`     | Same bounding box           | `geom ~= other_geom`   |

## Index-Aware Functions

These functions automatically use spatial indexes when available:

- `ST_Intersects(geom1, geom2)` - Tests if geometries intersect
- `ST_Contains(geom1, geom2)` - Tests if geom1 contains geom2
- `ST_Within(geom1, geom2)` - Tests if geom1 is within geom2
- `ST_DWithin(geom1, geom2, distance)` - Tests if geometries are within distance

## Performance Tips

### 1. Always Create Spatial Indexes
```sql
-- Good: Uses spatial index
CREATE INDEX my_table_geom_idx ON my_table USING GIST (geom gist_geometry_ops_simple);

-- Then queries like this will be fast:
SELECT * FROM my_table WHERE geom && ST_MakePoint(x, y);
```

### 2. Use Spatial Operators in WHERE Clauses
```sql
-- Good: Uses index
SELECT * FROM locations WHERE geom && ST_MakePoint(-122.4, 37.7);

-- Less optimal: May not use index efficiently
SELECT * FROM locations WHERE ST_Distance(geom, ST_MakePoint(-122.4, 37.7)) < 1000;
```

### 3. Combine Bounding Box and Exact Predicates
```sql
-- Optimal: First filter with &&, then exact test
SELECT * FROM locations 
WHERE geom && ST_MakePoint(-122.4, 37.7)  -- Uses index
  AND ST_DWithin(geom, ST_MakePoint(-122.4, 37.7), 1000);  -- Exact distance
```

## Testing Spatial Indexing

Run the included test script to verify spatial indexing works:

```bash
./test_spatial_indexing.sh
```

This script will:
1. Create a test database
2. Install RostGIS extension
3. Set up spatial indexing
4. Create test data
5. Create spatial indexes
6. Run spatial queries
7. Show query plans to verify index usage

## Troubleshooting

### Index Not Being Used?

Check if your query uses spatial operators:
```sql
-- This will use the index:
EXPLAIN ANALYZE SELECT * FROM locations WHERE geom && ST_MakePoint(-122.4, 37.7);

-- This might not:
EXPLAIN ANALYZE SELECT * FROM locations WHERE ST_Distance(geom, ST_MakePoint(-122.4, 37.7)) < 1000;
```

### Query Plan Shows Sequential Scan?

1. Make sure you have a spatial index:
```sql
\d+ locations  -- Should show indexes
```

2. Check if PostgreSQL thinks the index is worth using:
```sql
SET enable_seqscan = false;  -- Force index usage for testing
EXPLAIN ANALYZE SELECT ...;
SET enable_seqscan = true;   -- Reset
```

3. Update table statistics:
```sql
ANALYZE locations;
```

## Migrating from PostGIS

If you're migrating from PostGIS, most spatial queries should work with minimal changes:

```sql
-- PostGIS query:
SELECT * FROM locations WHERE ST_DWithin(geom, ST_MakePoint(-122.4, 37.7), 1000);

-- RostGIS equivalent (same query!):
SELECT * FROM locations WHERE ST_DWithin(geom, ST_MakePoint(-122.4, 37.7), 1000);
```

The main difference is in index creation:
```sql
-- PostGIS:
CREATE INDEX locations_geom_idx ON locations USING GIST (geom);

-- RostGIS:
CREATE INDEX locations_geom_idx ON locations USING GIST (geom gist_geometry_ops_simple);
```

## Limitations

Current limitations of the spatial indexing implementation:

1. **Simplified GiST functions**: Some internal GiST functions use simplified implementations
2. **Performance**: May not be as optimized as PostGIS for very large datasets
3. **Advanced features**: Some advanced PostGIS indexing features are not yet implemented

## Future Improvements

Planned enhancements for spatial indexing:

1. **Full GiST Implementation**: Complete GiST support function implementation in C
2. **SP-GiST Support**: Space-partitioned GiST indexes for better performance
3. **BRIN Support**: Block Range Indexes for very large, sorted datasets
4. **Index-only Scans**: Ability to answer queries from index data alone
5. **Advanced Statistics**: Better query planning with spatial statistics

## Getting Help

If you encounter issues with spatial indexing:

1. Check the [GitHub Issues](https://github.com/your-repo/rostgis/issues)
2. Run the test script to verify basic functionality
3. Enable query logging to see what queries are being executed
4. Check PostgreSQL logs for any errors

For performance issues, please include:
- Table sizes
- Query plans (`EXPLAIN ANALYZE`)
- Index definitions
- Sample queries 