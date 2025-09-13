use criterion::{black_box, criterion_group, criterion_main, Criterion};
fn bench_cost_estimation(c: &mut Criterion) {
    c.bench_function("cost_estimation", |b| {
        b.iter(|| {
            let result = black_box(42);
            result
        })
    });
}
criterion_group!(benches, bench_cost_estimation);
criterion_main!(benches);
