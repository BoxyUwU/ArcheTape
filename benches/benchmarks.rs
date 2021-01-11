use criterion::*;

mod bench_dynamic;
mod bench_static;

pub fn arche_tape(c: &mut Criterion) {
    let mut group = c.benchmark_group("arche_tape");

    group.bench_function("static_frag_iter_20_padding_20", |b| {
        let mut bench = bench_static::frag_iter_20_padding_20::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("dynamic_frag_iter_20_padding_20", |b| {
        let mut bench = bench_dynamic::frag_iter_20_padding_20::Benchmark::new();
        b.iter(move || bench.run());
    });

    group.bench_function("simple_insert_10_000", |b| {
        let mut bench = bench_static::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_insert_10_000_/_26", |b| {
        let mut bench = bench_static::frag_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("add_remove_10_000", |b| {
        let mut bench = bench_static::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("padded_add_remove_10_000", |b| {
        let mut bench = bench_static::padded_add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("wide_remove_10_000", |b| {
        let mut bench = bench_static::wide_add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("padded_get", |b| {
        let mut bench = bench_static::padded_get::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("get", |b| {
        let mut bench = bench_static::get::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("simple_iter", |b| {
        let mut bench = bench_static::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_iter_20_entity", |b| {
        let mut bench = bench_static::frag_iter_20::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_iter_200_entity", |b| {
        let mut bench = bench_static::frag_iter_200::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("frag_iter_2000_entity", |b| {
        let mut bench = bench_static::frag_iter_2000::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("simple_large_iter", |b| {
        let mut bench = bench_static::simple_large_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
}

criterion_group!(benchmarks, arche_tape);

criterion_main!(benchmarks);
