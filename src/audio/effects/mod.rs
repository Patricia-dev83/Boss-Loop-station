//! Audio effects processing
pub mod reverb;
pub mod delay;
pub mod compressor;
pub mod pitch;

// src/audio/effects/mod.rs
pub struct EffectsChain {
    pub effects: Vec<Box<dyn AudioEffect>>,
    pub enabled: bool,
}

impl EffectsChain {
    pub fn process(&mut self, buffer: &mut AudioBuffer) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        for effect in &mut self.effects {
            effect.process(buffer)?;
        }
        
        Ok(())
    }
}