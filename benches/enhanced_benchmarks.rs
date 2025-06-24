use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rostgis::functions::*;
use rostgis::spatial_index::{GeometryWithId, SpatialIndex};
use rostgis::vectorized_ops::VectorizedOps;

fn bench_spatial_indexing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_indexing");

    // Create test data: 1000 random points
    let mut geom_with_ids = Vec::new();

    for i in 0..1000 {
        let x = (i as f64 * 1.123) % 100.0;
        let y = (i as f64 * 2.456) % 100.0;
        let geom = make_point(x, y);
        geom_with_ids.push(GeometryWithId::new(i as i64, geom));
    }

    // Benchmark R*-tree creation
    group.bench_function("rtree_creation", |b| {
        b.iter(|| SpatialIndex::from_geometries(black_box(geom_with_ids.clone())))
    });

    // Create index for query benchmarks
    let index = SpatialIndex::from_geometries(geom_with_ids.clone());

    // Benchmark nearest neighbor queries
    group.bench_function("nearest_neighbor_query", |b| {
        b.iter(|| index.nearest_neighbor(black_box([50.0, 50.0])))
    });

    // Benchmark k-nearest neighbors
    group.bench_function("k_nearest_neighbors_10", |b| {
        b.iter(|| index.k_nearest_neighbors(black_box([50.0, 50.0]), black_box(10)))
    });

    // Benchmark range queries
    group.bench_function("range_query", |b| {
        b.iter(|| {
            use rostgis::spatial_index::BBox;
            let bbox = BBox::new(40.0, 40.0, 60.0, 60.0);
            index.query_bbox(black_box(&bbox))
        })
    });

    // Benchmark within distance queries
    group.bench_function("within_distance_query", |b| {
        b.iter(|| index.within_distance(black_box([50.0, 50.0]), black_box(10.0)))
    });

    group.finish();
}

fn bench_vectorized_vs_single_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vectorized_vs_single");

    // Create test data: varying sizes to show scalability
    for size in [10, 100, 1000].iter() {
        let mut points1 = Vec::new();
        let mut points2 = Vec::new();
        let mut polygons = Vec::new();

        for i in 0..*size {
            let x1 = (i as f64 * 1.123) % 100.0;
            let y1 = (i as f64 * 2.456) % 100.0;
            let x2 = (i as f64 * 3.789) % 100.0;
            let y2 = (i as f64 * 4.012) % 100.0;

            points1.push(make_point(x1, y1));
            points2.push(make_point(x2, y2));

            // Create small polygons for area calculations (fixed: proper 5-point polygon)
            let poly_wkt = format!(
                "POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))",
                x1,
                y1,
                x1 + 1.0,
                y1,
                x1 + 1.0,
                y1 + 1.0,
                x1,
                y1 + 1.0,
                x1,
                y1
            );
            if let Ok(poly) = geometry_from_wkt(&poly_wkt) {
                polygons.push(poly);
            }
        }

        // Benchmark single-operation distance calculations
        group.bench_with_input(
            BenchmarkId::new("single_distances", size),
            &(*size, &points1, &points2),
            |b, (_, p1, p2)| {
                b.iter(|| {
                    let mut distances = Vec::new();
                    for (pt1, pt2) in p1.iter().zip(p2.iter()) {
                        distances.push(geometries_distance(
                            black_box(pt1.clone()),
                            black_box(pt2.clone()),
                        ));
                    }
                    distances
                })
            },
        );

        // Benchmark vectorized distance calculations
        group.bench_with_input(
            BenchmarkId::new("vectorized_distances", size),
            &(*size, &points1, &points2),
            |b, (_, p1, p2)| {
                b.iter(|| {
                    VectorizedOps::bulk_distance_calculation(
                        black_box(p1.to_vec()),
                        black_box(p2.to_vec()),
                    )
                })
            },
        );

        // Benchmark single-operation area calculations
        group.bench_with_input(
            BenchmarkId::new("single_areas", size),
            &(*size, &polygons),
            |b, (_, polys)| {
                b.iter(|| {
                    let mut areas = Vec::new();
                    for poly in polys.iter() {
                        areas.push(geometry_area(black_box(poly.clone())));
                    }
                    areas
                })
            },
        );

        // Benchmark vectorized area calculations
        group.bench_with_input(
            BenchmarkId::new("vectorized_areas", size),
            &(*size, &polygons),
            |b, (_, polys)| {
                b.iter(|| VectorizedOps::bulk_area_calculation(black_box(polys.to_vec())))
            },
        );
    }

    group.finish();
}

fn bench_spatial_operations_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_operations_scaling");

    // Test how operations scale with data size
    for size in [100, 500, 1000, 2000, 5000].iter() {
        let mut geometries = Vec::new();

        for i in 0..*size {
            let x = (i as f64 * 1.123) % 1000.0;
            let y = (i as f64 * 2.456) % 1000.0;
            geometries.push(make_point(x, y));
        }

        // Benchmark bounding box calculations
        group.bench_with_input(
            BenchmarkId::new("bulk_bboxes", size),
            &geometries,
            |b, geoms| b.iter(|| VectorizedOps::bulk_bounding_boxes(black_box(geoms.clone()))),
        );

        // Create spatial index and benchmark bulk operations
        let geom_with_ids: Vec<GeometryWithId> = geometries
            .iter()
            .enumerate()
            .map(|(i, geom)| GeometryWithId::new(i as i64, geom.clone()))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("spatial_index_bulk_load", size),
            &geom_with_ids,
            |b, geoms| b.iter(|| SpatialIndex::from_geometries(black_box(geoms.clone()))),
        );
    }

    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Create large dataset to test memory usage patterns
    let large_dataset_size = 10000;
    let mut geometries = Vec::new();

    for i in 0..large_dataset_size {
        let x = (i as f64 * 1.123) % 10000.0;
        let y = (i as f64 * 2.456) % 10000.0;
        geometries.push(make_point(x, y));
    }

    // Benchmark memory-efficient bulk processing
    group.bench_function("bulk_stats_processing", |b| {
        b.iter(|| {
            use rostgis::vectorized_ops::bulk_geometry_stats;
            bulk_geometry_stats(black_box(geometries.clone()))
        })
    });

    // Test chunked processing for very large datasets
    group.bench_function("chunked_processing", |b| {
        b.iter(|| {
            let chunk_size = 1000;
            let mut total_results = Vec::new();

            for chunk in geometries.chunks(chunk_size) {
                let chunk_results = VectorizedOps::bulk_bounding_boxes(black_box(chunk.to_vec()));
                total_results.extend(chunk_results);
            }
            total_results
        })
    });

    group.finish();
}

criterion_group!(
    enhanced_benches,
    bench_spatial_indexing_performance,
    bench_vectorized_vs_single_operations,
    bench_spatial_operations_scaling,
    bench_memory_efficiency
);

criterion_main!(enhanced_benches);
