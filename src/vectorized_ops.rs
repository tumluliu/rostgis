use crate::geometry::Geometry;
use pgrx::prelude::*;

/// Vectorized geometry operations for bulk processing
/// This provides significant performance improvements for large datasets
pub struct VectorizedOps;

impl VectorizedOps {
    /// Convert a vector of RostGIS geometries to simplified format for processing
    pub fn prepare_for_bulk_processing(geometries: Vec<Geometry>) -> Vec<(f64, f64)> {
        let mut coordinates = Vec::new();

        for geom in geometries {
            match geom {
                Geometry::Point(point, _) => {
                    coordinates.push((point.x(), point.y()));
                }
                _ => {
                    // For other geometries, use centroid of bounding box
                    let (min_x, min_y, max_x, max_y) = geom.bounding_box();
                    let center_x = (min_x + max_x) / 2.0;
                    let center_y = (min_y + max_y) / 2.0;
                    coordinates.push((center_x, center_y));
                }
            }
        }

        coordinates
    }

    /// Bulk distance calculation using vectorized operations
    pub fn bulk_distance_calculation(points1: Vec<Geometry>, points2: Vec<Geometry>) -> Vec<f64> {
        points1
            .into_iter()
            .zip(points2.into_iter())
            .map(|(p1, p2)| match (p1, p2) {
                (Geometry::Point(pt1, _), Geometry::Point(pt2, _)) => {
                    let dx = pt1.x() - pt2.x();
                    let dy = pt1.y() - pt2.y();
                    (dx * dx + dy * dy).sqrt()
                }
                _ => 0.0,
            })
            .collect()
    }

    /// Bulk area calculation for polygons using vectorized operations
    pub fn bulk_area_calculation(polygons: Vec<Geometry>) -> Vec<f64> {
        use geo::Area;

        polygons
            .into_iter()
            .map(|geom| match geom {
                Geometry::Polygon(poly, _) => poly.unsigned_area(),
                Geometry::MultiPolygon(multipoly, _) => multipoly.unsigned_area(),
                _ => 0.0,
            })
            .collect()
    }

    /// Bulk bounding box calculation
    pub fn bulk_bounding_boxes(geometries: Vec<Geometry>) -> Vec<(f64, f64, f64, f64)> {
        geometries
            .into_iter()
            .map(|geom| geom.bounding_box())
            .collect()
    }

    /// Vectorized spatial predicate testing (e.g., contains, intersects)
    pub fn bulk_spatial_predicates(
        geometries1: Vec<Geometry>,
        geometries2: Vec<Geometry>,
        predicate: SpatialPredicate,
    ) -> Vec<bool> {
        geometries1
            .into_iter()
            .zip(geometries2.into_iter())
            .map(|(g1, g2)| match predicate {
                SpatialPredicate::Overlaps => g1.bbox_overlaps(&g2),
                SpatialPredicate::Contains => g1.bbox_contains(&g2),
                SpatialPredicate::Within => g1.bbox_within(&g2),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpatialPredicate {
    Overlaps,
    Contains,
    Within,
}

/// PostgreSQL function for bulk distance calculations
#[pg_extern(immutable, parallel_safe)]
pub fn bulk_distances(points1: Vec<Geometry>, points2: Vec<Geometry>) -> Vec<f64> {
    VectorizedOps::bulk_distance_calculation(points1, points2)
}

/// PostgreSQL function for bulk area calculations
#[pg_extern(immutable, parallel_safe)]
pub fn bulk_areas(polygons: Vec<Geometry>) -> Vec<f64> {
    VectorizedOps::bulk_area_calculation(polygons)
}

/// PostgreSQL function for bulk bounding box calculations
#[pg_extern(immutable, parallel_safe)]
pub fn bulk_bboxes(geometries: Vec<Geometry>) -> Vec<String> {
    VectorizedOps::bulk_bounding_boxes(geometries)
        .into_iter()
        .map(|(min_x, min_y, max_x, max_y)| format!("BOX({} {},{} {})", min_x, min_y, max_x, max_y))
        .collect()
}

/// PostgreSQL function for bulk spatial overlap testing
#[pg_extern(immutable, parallel_safe)]
pub fn bulk_overlaps(geometries1: Vec<Geometry>, geometries2: Vec<Geometry>) -> Vec<bool> {
    VectorizedOps::bulk_spatial_predicates(geometries1, geometries2, SpatialPredicate::Overlaps)
}

/// PostgreSQL function for bulk spatial contains testing
#[pg_extern(immutable, parallel_safe)]
pub fn bulk_contains(geometries1: Vec<Geometry>, geometries2: Vec<Geometry>) -> Vec<bool> {
    VectorizedOps::bulk_spatial_predicates(geometries1, geometries2, SpatialPredicate::Contains)
}

/// Performance-optimized bulk geometry processing with statistics
#[pg_extern(immutable, parallel_safe)]
pub fn bulk_geometry_stats(geometries: Vec<Geometry>) -> String {
    let start_time = std::time::Instant::now();

    let total_count = geometries.len();
    let areas = VectorizedOps::bulk_area_calculation(geometries.clone());
    let _bboxes = VectorizedOps::bulk_bounding_boxes(geometries);

    let total_area: f64 = areas.iter().sum();
    let avg_area = if total_count > 0 {
        total_area / total_count as f64
    } else {
        0.0
    };

    let processing_time = start_time.elapsed();

    format!(
        "Processed {} geometries in {:?}\nTotal area: {:.2}\nAverage area: {:.2}\nThroughput: {:.0} geom/sec",
        total_count,
        processing_time,
        total_area,
        avg_area,
        total_count as f64 / processing_time.as_secs_f64()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::make_point;

    #[test]
    fn test_bulk_distances() {
        let points1 = vec![make_point(0.0, 0.0), make_point(1.0, 1.0)];
        let points2 = vec![make_point(3.0, 4.0), make_point(4.0, 5.0)];

        let distances = VectorizedOps::bulk_distance_calculation(points1, points2);
        assert_eq!(distances.len(), 2);
        assert!((distances[0] - 5.0).abs() < 1e-10); // Distance from (0,0) to (3,4)
    }

    #[test]
    fn test_bulk_areas() {
        use crate::functions::geometry_from_wkt;

        let polygons = vec![
            geometry_from_wkt("POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))").unwrap(),
            geometry_from_wkt("POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))").unwrap(),
        ];

        let areas = VectorizedOps::bulk_area_calculation(polygons);
        assert_eq!(areas.len(), 2);
        assert!((areas[0] - 1.0).abs() < 1e-10); // 1x1 square
        assert!((areas[1] - 4.0).abs() < 1e-10); // 2x2 square
    }
}
