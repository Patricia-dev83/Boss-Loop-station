//! Master clock implementation

// src/sync/clock.rs
pub struct MasterClock {
    bpm: Arc<Mutex<f32>>,
    sample_rate: u32,
    beat_counter: AtomicUsize,
}

impl MasterClock {
    pub fn get_position(&self) -> (usize, f32) {
        let samples_per_beat = self.samples_per_beat();
        let total_samples = self.beat_counter.load(Ordering::Relaxed);
        let current_beat = total_samples / samples_per_beat;
        let beat_progress = (total_samples % samples_per_beat) as f32 / samples_per_beat as f32;
        (current_beat, beat_progress)
    }

    pub fn samples_per_beat(&self) -> usize {
        let bpm = self.bpm.lock().unwrap();
        ((60.0 / *bpm) * self.sample_rate as f32) as usize
    }
}