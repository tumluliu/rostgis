use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rostgis::functions::*;
use rostgis::geometry::Geometry;

fn bench_point_creation(c: &mut Criterion) {
    c.bench_function("make_point", |b| {
        b.iter(|| make_point(black_box(1.0), black_box(2.0)))
    });
}

fn bench_wkt_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("wkt_parsing");

    let point_wkt = "POINT(1 2)";
    let linestring_wkt = "LINESTRING(0 0, 1 1, 2 2)";
    let polygon_wkt = "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))";

    group.bench_with_input(BenchmarkId::new("point", "simple"), &point_wkt, |b, wkt| {
        b.iter(|| geometry_from_wkt(black_box(wkt)))
    });

    group.bench_with_input(
        BenchmarkId::new("linestring", "simple"),
        &linestring_wkt,
        |b, wkt| b.iter(|| geometry_from_wkt(black_box(wkt))),
    );

    group.bench_with_input(
        BenchmarkId::new("polygon", "simple"),
        &polygon_wkt,
        |b, wkt| b.iter(|| geometry_from_wkt(black_box(wkt))),
    );

    group.finish();
}

fn bench_geometry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry_operations");

    let point1 = make_point(0.0, 0.0);
    let point2 = make_point(3.0, 4.0);

    group.bench_function("distance_calculation", |b| {
        b.iter(|| geometries_distance(black_box(point1.clone()), black_box(point2.clone())))
    });

    let polygon = geometry_from_wkt("POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))").unwrap();

    group.bench_function("area_calculation", |b| {
        b.iter(|| geometry_area(black_box(polygon.clone())))
    });

    group.bench_function("perimeter_calculation", |b| {
        b.iter(|| geometry_perimeter(black_box(polygon.clone())))
    });

    group.finish();
}

fn bench_wkt_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("wkt_output");

    let point = make_point(1.0, 2.0);
    let linestring = geometry_from_wkt("LINESTRING(0 0, 1 1, 2 2)").unwrap();
    let polygon = geometry_from_wkt("POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))").unwrap();

    group.bench_function("point_to_wkt", |b| {
        b.iter(|| geometry_as_text(black_box(point.clone())))
    });

    group.bench_function("linestring_to_wkt", |b| {
        b.iter(|| geometry_as_text(black_box(linestring.clone())))
    });

    group.bench_function("polygon_to_wkt", |b| {
        b.iter(|| geometry_as_text(black_box(polygon.clone())))
    });

    group.finish();
}

fn bench_geojson_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("geojson_output");

    let point = make_point(1.0, 2.0);
    let linestring = geometry_from_wkt("LINESTRING(0 0, 1 1, 2 2)").unwrap();
    let polygon = geometry_from_wkt("POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))").unwrap();

    group.bench_function("point_to_geojson", |b| {
        b.iter(|| geometry_as_geojson(black_box(point.clone())))
    });

    group.bench_function("linestring_to_geojson", |b| {
        b.iter(|| geometry_as_geojson(black_box(linestring.clone())))
    });

    group.bench_function("polygon_to_geojson", |b| {
        b.iter(|| geometry_as_geojson(black_box(polygon.clone())))
    });

    group.finish();
}

fn bench_srid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("srid_operations");

    let point = make_point(1.0, 2.0);

    group.bench_function("get_srid", |b| {
        b.iter(|| geometry_srid(black_box(point.clone())))
    });

    group.bench_function("set_srid", |b| {
        b.iter(|| set_geometry_srid(black_box(point.clone()), black_box(4326)))
    });

    group.finish();
}

fn bench_comparison_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_operations");

    let point1 = make_point(1.0, 2.0);
    let point2 = make_point(1.0, 2.0);
    let point3 = make_point(3.0, 4.0);

    group.bench_function("equal_geometries", |b| {
        b.iter(|| geometries_equal(black_box(point1.clone()), black_box(point2.clone())))
    });

    group.bench_function("different_geometries", |b| {
        b.iter(|| geometries_equal(black_box(point1.clone()), black_box(point3.clone())))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_point_creation,
    bench_wkt_parsing,
    bench_geometry_operations,
    bench_wkt_output,
    bench_geojson_output,
    bench_srid_operations,
    bench_comparison_operations
);

criterion_main!(benches);
