use criterion::{black_box, criterion_group, criterion_main, Criterion};
use databases::usb::UsbCache;

pub fn criterion_benchmark(c: &mut Criterion) {
    let cache = UsbCache::new();
    c.bench_function("10 device USB lookup", |b| {
        b.iter(|| {
            let test_ids = black_box([
                "USB\\VID_068E&PID_00F3\\5&DD4B79A&0&5",
                "USB\\VID_131D&PID_0158\\311830",
                "USB\\VID_046D&PID_C092&MI_01\\7&384A91C&1&0001",
                "USB\\VID_046D&PID_C092\\2082346D4746",
                "USB\\VID_044F&PID_B687\\6&36992B61&0&2",
                "USB\\VID_046D&PID_C336&MI_01\\7&20BE3E95&0&0001",
                "USB\\VID_046D&PID_C336&MI_00\\7&20BE3E95&0&0000",
                "USB\\VID_214B&PID_7250\\5&DD4B79A&0&1",
                "USB\\VID_046D&PID_C336\\138532663638",
                "USB\\VID_2833&PID_0083\\1WMHHA6C991512",
            ]);
            for id in test_ids {
                cache.find(id).unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
