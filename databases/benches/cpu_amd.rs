use criterion::{black_box, criterion_group, criterion_main, Criterion};
use databases::cpu::CpuCache;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut cache = CpuCache::new();

    c.bench_function("5 CPU AMD lookup (no repeats)", |b| {
        b.iter(|| {
            let test_cpus = black_box([
                "AMD Ryzen 5 3400G with Radeon Vega Graphics",
                "AMD Ryzen 5 PRO 4650G with Radeon Graphics",
                "AMD Ryzen 5 5600 6-Core Processor",
                "AMD Ryzen 5 2600 Six-Core Processor",
                "AMD Ryzen 5 7600 6-Core Processor",
            ]);
            for id in test_cpus {
                cache.find(id).unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
