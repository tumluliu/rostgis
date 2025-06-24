# Basic Spatial Queries Tutorial

Learn the fundamentals of spatial queries with RostGIS through practical examples.

## Tutorial Overview

This tutorial covers:
1. Setting up spatial data
2. Creating points and geometries
3. Basic spatial queries
4. Using spatial indexes
5. Common query patterns

## Prerequisites

- RostGIS extension installed
- Basic SQL knowledge
- PostgreSQL database access

## Step 1: Setup and Sample Data

### Create a Test Database
```sql
-- Connect to PostgreSQL
CREATE DATABASE spatial_tutorial;
\c spatial_tutorial

-- Install RostGIS
CREATE EXTENSION rostgis;

-- Verify installation
SELECT rostgis_version();
```

### Set Up Spatial Indexing
```sql
-- Run spatial indexing setup (required for indexes)
\i sql/gist_index_setup.sql
```

### Create Sample Tables
```sql
-- Cities table
CREATE TABLE cities (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    country TEXT,
    population INTEGER,
    location GEOMETRY
);

-- Points of interest table
CREATE TABLE poi (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT,
    location GEOMETRY
);

-- Road network table
CREATE TABLE roads (
    id SERIAL PRIMARY KEY,
    name TEXT,
    type TEXT,
    geometry GEOMETRY
);
```

## Step 2: Creating Spatial Data

### Insert City Data
```sql
-- Major world cities with coordinates
INSERT INTO cities (name, country, population, location) VALUES
    ('San Francisco', 'USA', 874961, ST_MakePoint(-122.4194, 37.7749)),
    ('New York', 'USA', 8336817, ST_MakePoint(-74.0060, 40.7128)),
    ('London', 'UK', 8982000, ST_MakePoint(-0.1276, 51.5074)),
    ('Tokyo', 'Japan', 13929286, ST_MakePoint(139.6503, 35.6762)),
    ('Sydney', 'Australia', 5312163, ST_MakePoint(151.2093, -33.8688)),
    ('Paris', 'France', 2161000, ST_MakePoint(2.3522, 48.8566)),
    ('Berlin', 'Germany', 3669491, ST_MakePoint(13.4050, 52.5200));

-- Set SRID to WGS84 (GPS coordinates)
UPDATE cities SET location = ST_SetSRID(location, 4326);
```

### Insert Points of Interest
```sql
INSERT INTO poi (name, category, location) VALUES
    ('Golden Gate Bridge', 'landmark', ST_SetSRID(ST_MakePoint(-122.4783, 37.8199), 4326)),
    ('Central Park', 'park', ST_SetSRID(ST_MakePoint(-73.9657, 40.7829), 4326)),
    ('Tower Bridge', 'landmark', ST_SetSRID(ST_MakePoint(-0.0754, 51.5055), 4326)),
    ('Tokyo Tower', 'landmark', ST_SetSRID(ST_MakePoint(139.7454, 35.6586), 4326)),
    ('Opera House', 'landmark', ST_SetSRID(ST_MakePoint(151.2153, -33.8568), 4326)),
    ('Eiffel Tower', 'landmark', ST_SetSRID(ST_MakePoint(2.2945, 48.8584), 4326));
```

### Create Linear Data
```sql
-- Simple road segments
INSERT INTO roads (name, type, geometry) VALUES
    ('Highway 101', 'highway', 
     ST_SetSRID(ST_GeomFromText('LINESTRING(-122.5 37.7, -122.4 37.8, -122.3 37.9)'), 4326)),
    ('Broadway', 'street',
     ST_SetSRID(ST_GeomFromText('LINESTRING(-73.97 40.76, -73.98 40.77, -73.99 40.78)'), 4326));
```

## Step 3: Basic Geometry Queries

### Viewing Spatial Data
```sql
-- See all cities with coordinates
SELECT name, country, ST_AsText(location) as coordinates
FROM cities
ORDER BY name;

-- Get coordinates as separate columns
SELECT name, 
       ST_X(location) as longitude,
       ST_Y(location) as latitude
FROM cities;

-- Export as GeoJSON
SELECT name, ST_AsGeoJSON(location) as geojson
FROM cities
WHERE country = 'USA';
```

### Geometry Properties
```sql
-- Check geometry types
SELECT name, ST_GeometryType(location) as geom_type
FROM cities;

-- Check spatial reference systems
SELECT name, ST_SRID(location) as srid
FROM cities;

-- Get bounding boxes
SELECT name, ST_Envelope(location) as bbox
FROM cities;
```

## Step 4: Distance and Proximity Queries

### Calculate Distances
```sql
-- Distance between two specific cities
SELECT 
    ST_Distance(
        (SELECT location FROM cities WHERE name = 'San Francisco'),
        (SELECT location FROM cities WHERE name = 'New York')
    ) as distance_degrees;

-- All distances from San Francisco
SELECT name, 
       ST_Distance(location, 
                  (SELECT location FROM cities WHERE name = 'San Francisco')
       ) as distance_from_sf
FROM cities
WHERE name != 'San Francisco'
ORDER BY distance_from_sf;
```

### Find Nearby Features
```sql
-- Cities within 2 degrees of London
SELECT name, country,
       ST_Distance(location, 
                  (SELECT location FROM cities WHERE name = 'London')
       ) as distance
FROM cities
WHERE ST_DWithin(location, 
                 (SELECT location FROM cities WHERE name = 'London'), 
                 2.0)
  AND name != 'London'
ORDER BY distance;

-- Points of interest near cities
SELECT c.name as city, p.name as poi, p.category,
       ST_Distance(c.location, p.location) as distance
FROM cities c, poi p
WHERE ST_DWithin(c.location, p.location, 0.1)
ORDER BY c.name, distance;
```

## Step 5: Creating and Using Spatial Indexes

### Create Spatial Indexes
```sql
-- Create spatial indexes for better performance
CREATE INDEX cities_location_idx ON cities 
USING GIST (location gist_geometry_ops_simple);

CREATE INDEX poi_location_idx ON poi 
USING GIST (location gist_geometry_ops_simple);

CREATE INDEX roads_geometry_idx ON roads 
USING GIST (geometry gist_geometry_ops_simple);
```

### Verify Index Usage
```sql
-- Check if index is being used
EXPLAIN ANALYZE 
SELECT name FROM cities 
WHERE location && ST_MakePoint(-122.4, 37.7);

-- Should show "Index Scan using cities_location_idx"
```

### Index-Accelerated Queries
```sql
-- Spatial overlap queries (use && operator)
SELECT name FROM cities 
WHERE location && ST_MakePoint(-122.4, 37.7);

-- Bounding box queries
SELECT c.name, p.name
FROM cities c, poi p
WHERE c.location && p.location;

-- Containment queries
SELECT name FROM cities
WHERE location @ ST_GeomFromText('POLYGON((-125 30, -65 30, -65 50, -125 50, -125 30))');
```

## Step 6: Common Query Patterns

### Spatial Joins
```sql
-- Cities and their nearby points of interest
SELECT DISTINCT c.name as city, p.name as poi
FROM cities c
JOIN poi p ON ST_DWithin(c.location, p.location, 0.5)
ORDER BY c.name;

-- Count POIs near each city
SELECT c.name, c.country, COUNT(p.id) as poi_count
FROM cities c
LEFT JOIN poi p ON ST_DWithin(c.location, p.location, 0.5)
GROUP BY c.id, c.name, c.country
ORDER BY poi_count DESC;
```

### Spatial Aggregation
```sql
-- Find the center point of all cities
SELECT ST_AsText(
    ST_MakePoint(
        AVG(ST_X(location)),
        AVG(ST_Y(location))
    )
) as center_point
FROM cities;

-- Cities by continent (rough grouping by longitude)
SELECT 
    CASE 
        WHEN ST_X(location) < -30 THEN 'Americas'
        WHEN ST_X(location) < 60 THEN 'Europe/Africa'
        ELSE 'Asia/Pacific'
    END as region,
    COUNT(*) as city_count,
    AVG(population) as avg_population
FROM cities
GROUP BY region;
```

### Spatial Filtering
```sql
-- Cities in the Northern Hemisphere
SELECT name, country FROM cities
WHERE ST_Y(location) > 0;

-- Cities in a bounding box (roughly USA)
SELECT name, country FROM cities
WHERE location && ST_GeomFromText('POLYGON((-125 25, -65 25, -65 50, -125 50, -125 25))');

-- Filter by distance from a reference point
WITH reference_point AS (
    SELECT ST_MakePoint(0, 0) as point  -- Greenwich/Equator
)
SELECT name, country,
       ST_Distance(location, reference_point.point) as distance_from_origin
FROM cities, reference_point
WHERE ST_Distance(location, reference_point.point) < 100
ORDER BY distance_from_origin;
```

## Step 7: Advanced Query Examples

### Nearest Neighbor Queries
```sql
-- Find the 3 closest cities to a point
SELECT name, country,
       ST_Distance(location, ST_MakePoint(0, 50)) as distance
FROM cities
ORDER BY location <-> ST_MakePoint(0, 50)
LIMIT 3;

-- Closest POI to each city
SELECT DISTINCT ON (c.name) 
       c.name as city, 
       p.name as closest_poi,
       ST_Distance(c.location, p.location) as distance
FROM cities c
CROSS JOIN poi p
ORDER BY c.name, ST_Distance(c.location, p.location);
```

### Geometric Calculations
```sql
-- Calculate lengths of road segments
SELECT name, type,
       ST_Length(geometry) as length_degrees
FROM roads;

-- Create buffer zones around cities (conceptual - exact buffering planned)
SELECT name, 
       ST_AsText(location) as city_location,
       'Buffer zone around ' || name as description
FROM cities
WHERE population > 5000000;
```

### Data Validation
```sql
-- Check for valid geometries
SELECT name, ST_GeometryType(location) as type,
       CASE WHEN location IS NOT NULL THEN 'Valid' ELSE 'Invalid' END as status
FROM cities;

-- Find duplicate locations
SELECT ST_AsText(location), COUNT(*) as count
FROM cities
GROUP BY location
HAVING COUNT(*) > 1;

-- Check SRID consistency
SELECT DISTINCT ST_SRID(location) as srid, COUNT(*) as count
FROM cities
GROUP BY ST_SRID(location);
```

## Step 8: Performance Tips

### Query Optimization
```sql
-- Good: Use spatial indexes
SELECT name FROM cities 
WHERE location && ST_MakePoint(-122.4, 37.7);

-- Less optimal: Function that can't use index efficiently
SELECT name FROM cities 
WHERE ST_Distance(location, ST_MakePoint(-122.4, 37.7)) < 1;

-- Better: Combine spatial operator with distance
SELECT name FROM cities 
WHERE location && ST_MakePoint(-122.4, 37.7)
  AND ST_Distance(location, ST_MakePoint(-122.4, 37.7)) < 1;
```

### Index Usage
```sql
-- Update table statistics after loading data
ANALYZE cities;
ANALYZE poi;
ANALYZE roads;

-- Check index usage
EXPLAIN (ANALYZE, BUFFERS) 
SELECT name FROM cities 
WHERE location && ST_MakePoint(-122.4, 37.7);
```

## Summary

You've learned:
- ✅ Creating spatial tables and inserting geometry data
- ✅ Basic spatial queries and distance calculations  
- ✅ Creating and using spatial indexes
- ✅ Common spatial query patterns
- ✅ Performance optimization techniques

## Next Steps

1. **[Working with Points](WORKING_WITH_POINTS.md)** - Detailed point operations
2. **[Working with Polygons](WORKING_WITH_POLYGONS.md)** - Polygon analysis
3. **[Performance Optimization](PERFORMANCE_OPTIMIZATION.md)** - Advanced optimization
4. **[Real-world Examples](EXAMPLES.md)** - Complete applications

## Cleanup

```sql
-- Drop tutorial tables
DROP TABLE IF EXISTS roads;
DROP TABLE IF EXISTS poi;
DROP TABLE IF EXISTS cities;

-- Drop database (optional)
-- \c postgres
-- DROP DATABASE spatial_tutorial;
```

---

*This tutorial provides a foundation for spatial queries. For production use, consider coordinate reference systems, data validation, and error handling.* 