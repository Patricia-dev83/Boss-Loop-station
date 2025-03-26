use criterion::{black_box, criterion_group, criterion_main, Criterion};
use loop_station::audio::effects::Reverb;

fn bench_reverb(c: &mut Criterion) {
    let mut reverb = Reverb::new(44100);
    let mut buffer = vec![0.0; 1024];
    
    c.bench_function("reverb_1024_samples", |b| {
        b.iter(|| reverb.process(black_box(&mut buffer)))
    });
}

criterion_group!(benches, bench_reverb);
criterion_main!(benches);
