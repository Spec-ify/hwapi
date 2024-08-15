use criterion::{black_box, criterion_group, criterion_main, Criterion};
use databases::cpu::CpuCache;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut cache = CpuCache::new();

    c.bench_function("5 CPU Intel lookup (no cache)", |b| {
        b.iter(|| {
            let test_cpus = black_box([
                "Intel(R) Core(TM) i3-9100F CPU @ 3.60GHz",
                "Intel(R) Core(TM) i9-9900K CPU @ 3.60GHz",
                "Intel(R) Core(TM) i7-14700K",
                "Intel(R) Core(TM) i7 CPU M 620 @ 2.67Ghz",
                "Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz",
            ]);
            for id in test_cpus {
                cache.find(id).unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

