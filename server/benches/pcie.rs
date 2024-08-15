use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parsing::pcie::PcieCache;

pub fn criterion_benchmark(c: &mut Criterion) {
    let cache = PcieCache::new();
    c.bench_function("10 device PCIe lookup", |b| {
        b.iter(|| {
            let test_ids = black_box([
                "PCI\\VEN_8086&DEV_A123&SUBSYS_9C551019&REV_31\\3&11583659&0&FC",
                "PCI\\VEN_8086&DEV_A170&SUBSYS_9C551019&REV_31\\3&11583659&0&FB",
                "PCI\\VEN_8086&DEV_A170&SUBSYS_9C551019&REV_31\\3&11583659&0&FB",
                "PCI\\VEN_8086&DEV_A143&SUBSYS_9C551019&REV_31\\3&11583659&0&F8",
                "PCI\\VEN_8086&DEV_A117&SUBSYS_9C551019&REV_F1\\3&11583659&0&E7",
                "PCI\\VEN_10EC&DEV_8168&SUBSYS_9C551019&REV_0C\\4&6189617&0&00E0",
                "PCI\\VEN_8086&DEV_A115&SUBSYS_9C551019&REV_F1\\3&11583659&0&E0",
                "PCI\\VEN_8086&DEV_A102&SUBSYS_9C551019&REV_31\\3&11583659&0&B8",
                "PCI\\VEN_8086&DEV_A13A&SUBSYS_9C551019&REV_31\\3&11583659&0&B0",
                "PCI\\VEN_8086&DEV_A131&SUBSYS_9C551019&REV_31\\3&11583659&0&A2",
            ]);
            for id in test_ids {
                cache.find(id).unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
