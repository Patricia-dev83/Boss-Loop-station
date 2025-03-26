// benches/audio_processing.rs
#[bench]
fn bench_effect_chain(b: &mut Bencher) {
    let mut chain = EffectsChain::new();
    chain.add(Box::new(Reverb::new(44100)));
    
    let mut buffer = AudioBuffer::new(44100, 2);
    buffer.resize(4096);
    
    b.iter(|| {
        chain.process(&mut buffer).unwrap();
    });
}