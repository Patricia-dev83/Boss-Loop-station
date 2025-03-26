//! Audio buffer management

//! Advanced audio buffer management system
//!
//! Provides thread-safe, efficient audio buffer storage and processing
//! with features like:
//! - Multi-channel support
//! - Lock-free operations
//! - Efficient memory management
//! - DSP utilities

use std::{
    sync::Arc,
    ops::{Deref, DerefMut},
    time::Duration,
};
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use realfft::RealFftPlanner;
use crate::error::AudioError;

/// Main audio buffer structure with multi-channel support
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    samples: Arc<Vec<Vec<f32>>>,
    sample_rate: u32,
    channels: usize,
    capacity: usize,
}

/// Thread-safe buffer pool for efficient memory reuse
pub struct BufferPool {
    pool: DashMap<usize, SegQueue<Arc<Vec<Vec<f32>>>>>,
    max_buffers: usize,
}

/// Safe buffer handle with automatic pool return
pub struct PooledBuffer {
    data: Arc<Vec<Vec<f32>>>,
    pool: Arc<BufferPool>,
}

impl AudioBuffer {
    /// Create new empty buffer
    pub fn new(sample_rate: u32, channels: usize) -> Self {
        Self {
            samples: Arc::new(vec![Vec::new(); channels]),
            sample_rate,
            channels,
            capacity: 0,
        }
    }

    /// Create from existing data
    pub fn from_data(
        data: Vec<Vec<f32>>,
        sample_rate: u32,
    ) -> Result<Self, AudioError> {
        if data.is_empty() {
            return Err(AudioError::InvalidBuffer);
        }
        
        let channels = data.len();
        let capacity = data[0].capacity();
        
        Ok(Self {
            samples: Arc::new(data),
            sample_rate,
            channels,
            capacity,
        })
    }

    /// Get immutable reference to samples
    pub fn samples(&self) -> &[Vec<f32>] {
        &self.samples
    }

    /// Get mutable reference to samples (creates new Arc if needed)
    pub fn samples_mut(&mut self) -> &mut [Vec<f32>] {
        Arc::make_mut(&mut self.samples)
    }

    /// Get buffer length in samples
    pub fn len(&self) -> usize {
        self.samples[0].len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channel count
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Append samples to buffer (mono input)
    pub fn append_mono(&mut self, samples: &[f32]) {
        let samples = self.samples_mut();
        for channel in samples.iter_mut() {
            channel.extend_from_slice(samples);
        }
    }

    /// Append samples to buffer (multi-channel input)
    pub fn append(&mut self, samples: &[&[f32]]) -> Result<(), AudioError> {
        if samples.len() != self.channels {
            return Err(AudioError::ChannelMismatch);
        }

        let buffer = self.samples_mut();
        for (channel, new_samples) in buffer.iter_mut().zip(samples.iter()) {
            channel.extend_from_slice(new_samples);
        }
        Ok(())
    }

    /// Clear buffer contents
    pub fn clear(&mut self) {
        let samples = self.samples_mut();
        for channel in samples.iter_mut() {
            channel.clear();
        }
    }

    /// Mix another buffer into this one (with gain)
    pub fn mix(&mut self, other: &AudioBuffer, gain: f32) -> Result<(), AudioError> {
        if other.channels() != self.channels || other.sample_rate() != self.sample_rate {
            return Err(AudioError::BufferMismatch);
        }

        let target_len = self.len().max(other.len());
        self.resize(target_len);
        
        let src_samples = other.samples();
        let dst_samples = self.samples_mut();
        
        for (dst_channel, src_channel) in dst_samples.iter_mut().zip(src_samples.iter()) {
            for (dst_sample, src_sample) in dst_channel.iter_mut().zip(src_channel.iter()) {
                *dst_sample += *src_sample * gain;
            }
        }
        
        Ok(())
    }

    /// Resize buffer (padding with zeros if expanding)
    pub fn resize(&mut self, new_len: usize) {
        let samples = self.samples_mut();
        for channel in samples.iter_mut() {
            channel.resize(new_len, 0.0);
        }
    }

    /// Apply gain to entire buffer
    pub fn apply_gain(&mut self, gain: f32) {
        let samples = self.samples_mut();
        for channel in samples.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= gain;
            }
        }
    }

    /// Convert to mono by averaging channels
    pub fn to_mono(&mut self) {
        if self.channels == 1 {
            return;
        }

        let samples = self.samples_mut();
        let mono_data: Vec<f32> = samples[0]
            .iter()
            .enumerate()
            .map(|(i, _)| {
                samples.iter().map(|channel| channel[i]).sum::<f32>() / self.channels as f32
            })
            .collect();

        *samples = vec![mono_data];
        self.channels = 1;
    }
}

impl BufferPool {
    /// Create new buffer pool
    pub fn new(max_buffers: usize) -> Arc<Self> {
        Arc::new(Self {
            pool: DashMap::new(),
            max_buffers,
        })
    }

    /// Get buffer from pool or create new one
    pub fn get(&self, channels: usize, capacity: usize) -> PooledBuffer {
        let queue = self.pool.entry(channels).or_insert_with(|| SegQueue::new());
        
        if let Some(data) = queue.pop() {
            PooledBuffer {
                data,
                pool: Arc::new(self.clone()),
            }
        } else {
            PooledBuffer {
                data: Arc::new(vec![Vec::with_capacity(capacity); channels]),
                pool: Arc::new(self.clone()),
            }
        }
    }

    /// Return buffer to pool
    fn return_buffer(&self, data: Arc<Vec<Vec<f32>>>) {
        if self.pool.len() < self.max_buffers {
            let channels = data.len();
            if let Some(queue) = self.pool.get(&channels) {
                queue.push(data);
            }
        }
    }
}

impl PooledBuffer {
    /// Create new pooled buffer
    pub fn new(
        pool: Arc<BufferPool>,
        channels: usize,
        capacity: usize,
    ) -> Self {
        pool.get(channels, capacity)
    }

    /// Get length of buffer
    pub fn len(&self) -> usize {
        self.data[0].len()
    }
}

impl Deref for PooledBuffer {
    type Target = [Vec<f32>];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.data)
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        let data = std::mem::replace(&mut self.data, Arc::new(Vec::new()));
        self.pool.return_buffer(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_operations() {
        let mut buffer = AudioBuffer::new(44100, 2);
        buffer.append_mono(&[1.0, 2.0, 3.0]);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.samples()[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(buffer.samples()[1], vec![1.0, 2.0, 3.0]);

        buffer.apply_gain(0.5);
        assert_eq!(buffer.samples()[0], vec![0.5, 1.0, 1.5]);

        buffer.to_mono();
        assert_eq!(buffer.channels(), 1);
        assert_eq!(buffer.samples()[0], vec![0.5, 1.0, 1.5]);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(10);
        {
            let mut buffer = PooledBuffer::new(pool.clone(), 2, 1024);
            buffer[0].extend_from_slice(&[1.0, 2.0]);
            buffer[1].extend_from_slice(&[3.0, 4.0]);
        } // Buffer returned to pool here

        let buffer = PooledBuffer::new(pool.clone(), 2, 1024);
        assert!(buffer[0].capacity() >= 1024);
    }
}