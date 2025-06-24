# Migrating from PostGIS to RostGIS

This guide helps PostGIS users migrate to RostGIS, covering compatibility, differences, and migration strategies.

## Migration Overview

RostGIS is designed to be **PostGIS-compatible**, meaning most PostGIS functions and workflows should work with minimal or no changes. However, there are some differences to be aware of.

## Compatibility Matrix

### ✅ Fully Compatible Functions

These functions work identically to PostGIS:

| Function                 | PostGIS | RostGIS | Notes                     |
|--------------------------|---------|---------|---------------------------|
| `ST_MakePoint(x, y)`     | ✅       | ✅       | 100% compatible           |
| `ST_Point(x, y)`         | ✅       | ✅       | Alias for ST_MakePoint    |
| `ST_GeomFromText(wkt)`   | ✅       | ✅       | Full WKT support          |
| `ST_GeomFromWKT(wkt)`    | ✅       | ✅       | Alias for ST_GeomFromText |
| `ST_AsText(geom)`        | ✅       | ✅       | Identical output format   |
| `ST_AsWKT(geom)`         | ✅       | ✅       | Alias for ST_AsText       |
| `ST_X(geom)`             | ✅       | ✅       | Extract X coordinate      |
| `ST_Y(geom)`             | ✅       | ✅       | Extract Y coordinate      |
| `ST_SRID(geom)`          | ✅       | ✅       | Get spatial reference ID  |
| `ST_SetSRID(geom, srid)` | ✅       | ✅       | Set spatial reference ID  |
| `ST_GeometryType(geom)`  | ✅       | ✅       | Get geometry type         |
| `ST_Distance(g1, g2)`    | ✅       | ✅       | Euclidean distance        |
| `ST_Area(geom)`          | ✅       | ✅       | Calculate area            |
| `ST_Length(geom)`        | ✅       | ✅       | Calculate length          |
| `ST_Perimeter(geom)`     | ✅       | ✅       | Calculate perimeter       |
| `ST_Equals(g1, g2)`      | ✅       | ✅       | Test equality             |

### 🔄 Compatible with Differences

These functions work but may have minor differences:

| Function                | PostGIS | RostGIS | Differences                     |
|-------------------------|---------|---------|---------------------------------|
| `ST_Intersects(g1, g2)` | ✅       | ✅       | Uses bounding box optimization  |
| `ST_Contains(g1, g2)`   | ✅       | ✅       | Uses bounding box optimization  |
| `ST_Within(g1, g2)`     | ✅       | ✅       | Uses bounding box optimization  |
| `ST_DWithin(g1, g2, d)` | ✅       | ✅       | Simplified distance calculation |

### 🚧 Planned Functions

These functions are not yet implemented but are planned:

| Function                   | PostGIS | RostGIS | Status       |
|----------------------------|---------|---------|--------------|
| `ST_GeomFromWKB(wkb)`      | ✅       | 🚧      | Planned v0.2 |
| `ST_AsBinary(geom)`        | ✅       | 🚧      | Planned v0.2 |
| `ST_Buffer(geom, dist)`    | ✅       | 🚧      | Planned v0.3 |
| `ST_Union(g1, g2)`         | ✅       | 🚧      | Planned v0.3 |
| `ST_Intersection(g1, g2)`  | ✅       | 🚧      | Planned v0.3 |
| `ST_Difference(g1, g2)`    | ✅       | 🚧      | Planned v0.3 |
| `ST_Transform(geom, srid)` | ✅       | 🚧      | Planned v0.4 |

## Key Differences

### 1. Index Creation

**PostGIS:**
```sql
CREATE INDEX spatial_idx ON my_table USING GIST (geom);
```

**RostGIS:**
```sql
-- First run the spatial indexing setup (one time)
\i sql/gist_index_setup.sql

-- Then create indexes with the operator class
CREATE INDEX spatial_idx ON my_table USING GIST (geom gist_geometry_ops_simple);
```

### 2. Spatial Operators

Both systems support the same spatial operators, but RostGIS requires the setup script to be run first:

| Operator | Description | PostGIS | RostGIS |
|----------|-------------|---------|---------|
| `&&`     | Overlaps    | ✅       | ✅*      |
| `<<`     | Left of     | ✅       | ✅*      |
| `>>`     | Right of    | ✅       | ✅*      |
| `~`      | Contains    | ✅       | ✅*      |
| `@`      | Within      | ✅       | ✅*      |

*Requires spatial indexing setup script

### 3. Extension Installation

**PostGIS:**
```sql
CREATE EXTENSION postgis;
```

**RostGIS:**
```sql
CREATE EXTENSION rostgis;
```

## Migration Strategies

### Strategy 1: Side-by-Side Migration

Run PostGIS and RostGIS in parallel during migration:

1. **Install RostGIS** alongside existing PostGIS
2. **Create test schema** with RostGIS functions
3. **Validate results** against PostGIS
4. **Gradually migrate** table by table

```sql
-- Create separate schemas
CREATE SCHEMA postgis_data;
CREATE SCHEMA rostgis_data;

-- Install extensions in different schemas
CREATE EXTENSION postgis SCHEMA postgis_data;
CREATE EXTENSION rostgis SCHEMA rostgis_data;

-- Test compatibility
SELECT postgis_data.ST_AsText(postgis_data.ST_MakePoint(1, 2));
SELECT rostgis_data.ST_AsText(rostgis_data.ST_MakePoint(1, 2));
```

### Strategy 2: In-Place Migration

Replace PostGIS with RostGIS directly:

1. **Backup your database**
2. **Drop PostGIS extension**
3. **Install RostGIS extension**
4. **Update index definitions**
5. **Test all queries**

```sql
-- Backup first!
pg_dump your_database > backup.sql

-- Remove PostGIS
DROP EXTENSION postgis CASCADE;

-- Install RostGIS
CREATE EXTENSION rostgis;

-- Recreate spatial indexes with new syntax
-- (See index migration section below)
```

### Strategy 3: New Database Migration

Create a fresh database with RostGIS:

1. **Export data** without spatial functions
2. **Create new database** with RostGIS
3. **Import data** with updated spatial queries
4. **Switch applications** to new database

## Detailed Migration Steps

### Step 1: Assess Current Usage

Inventory your current PostGIS usage:

```sql
-- Find all PostGIS function usage
SELECT 
    routine_name,
    routine_type
FROM information_schema.routines 
WHERE routine_name LIKE 'st_%'
AND routine_schema = 'public';

-- Check spatial indexes
SELECT 
    indexname,
    indexdef
FROM pg_indexes 
WHERE indexdef LIKE '%GIST%'
AND indexdef LIKE '%geom%';
```

### Step 2: Test Compatibility

Create a compatibility test script:

```sql
-- Test basic functions
DO $$
DECLARE
    test_point GEOMETRY;
    postgis_result TEXT;
    rostgis_result TEXT;
BEGIN
    -- Test point creation and conversion
    test_point := ST_MakePoint(-122.4194, 37.7749);
    
    -- Compare results
    postgis_result := ST_AsText(test_point);
    -- Switch to RostGIS and test
    -- rostgis_result := ST_AsText(test_point);
    
    RAISE NOTICE 'Point creation test: %', postgis_result;
END $$;
```

### Step 3: Migrate Indexes

Convert PostGIS spatial indexes to RostGIS format:

```sql
-- PostGIS index
DROP INDEX IF EXISTS old_spatial_idx;

-- RostGIS index (after running setup script)
CREATE INDEX new_spatial_idx ON my_table 
USING GIST (geom gist_geometry_ops_simple);
```

### Step 4: Update Application Code

Most application code should work unchanged, but verify:

**Python (psycopg2):**
```python
# This should work the same with RostGIS
cursor.execute("""
    SELECT ST_AsText(ST_MakePoint(%s, %s))
""", (longitude, latitude))
```

**Node.js (pg):**
```javascript
// This should work the same with RostGIS
const result = await client.query(
    'SELECT ST_AsText(ST_MakePoint($1, $2))',
    [longitude, latitude]
);
```

### Step 5: Performance Testing

Compare performance before and after migration:

```sql
-- Test query performance
EXPLAIN ANALYZE
SELECT COUNT(*) 
FROM locations 
WHERE geom && ST_MakePoint(-122.4, 37.7);
```

## Common Migration Issues

### Issue 1: Index Not Found Error

**Error:** `operator class "gist_geometry_ops" does not exist`

**Solution:** Run the spatial indexing setup script:
```sql
\i sql/gist_index_setup.sql
```

### Issue 2: Function Not Found

**Error:** `function st_intersects(geometry, geometry) does not exist`

**Solution:** Verify RostGIS extension is installed:
```sql
SELECT * FROM pg_extension WHERE extname = 'rostgis';
```

### Issue 3: Performance Regression

**Problem:** Queries are slower after migration

**Solutions:**
1. Ensure spatial indexes are created with correct operator class
2. Run `ANALYZE` on tables after migration
3. Check query plans with `EXPLAIN ANALYZE`

### Issue 4: Different Results

**Problem:** Spatial functions return different results

**Analysis:** 
- RostGIS uses bounding box optimization for some functions
- For exact compatibility, use PostGIS until RostGIS implements exact algorithms

## Rollback Plan

If migration doesn't work as expected:

### Immediate Rollback
```sql
-- Drop RostGIS
DROP EXTENSION rostgis CASCADE;

-- Reinstall PostGIS
CREATE EXTENSION postgis;

-- Restore from backup if needed
psql your_database < backup.sql
```

### Data-Only Rollback
```sql
-- Export data without spatial functions
\copy (SELECT id, name, ST_AsText(geom) as geom_wkt FROM locations) TO 'data.csv';

-- Recreate with PostGIS
-- Import data back
```

## Performance Comparison

Expected performance characteristics:

| Operation            | PostGIS | RostGIS    | Notes                         |
|----------------------|---------|------------|-------------------------------|
| Point creation       | Fast    | Faster     | 3.39M ops/sec in RostGIS      |
| Distance calculation | Fast    | Faster     | 3.92M ops/sec in RostGIS      |
| WKT parsing          | Fast    | Comparable | 0.87-2.74M ops/sec            |
| Complex operations   | Mature  | Developing | PostGIS more feature-complete |

## Migration Checklist

- [ ] **Backup current database**
- [ ] **Inventory PostGIS function usage**
- [ ] **Test RostGIS compatibility**
- [ ] **Plan index migration strategy**
- [ ] **Test performance in staging environment**
- [ ] **Update application connection strings**
- [ ] **Migrate indexes with new syntax**
- [ ] **Validate query results**
- [ ] **Monitor performance after migration**
- [ ] **Document any differences found**

## Getting Help

For migration assistance:

1. **Review compatibility matrix** above
2. **Test in staging environment** first
3. **Search existing GitHub issues** for migration problems
4. **Create GitHub issue** with specific migration questions
5. **Join community discussions** for migration tips

## Future Roadmap

RostGIS compatibility improvements planned:

- **v0.2**: WKB support, exact geometric predicates
- **v0.3**: Geometric operations (buffer, union, intersection)
- **v0.4**: Coordinate system transformations
- **v0.5**: Full PostGIS function compatibility

---

*This migration guide is updated with each RostGIS release. Check the latest version for current compatibility status.* 