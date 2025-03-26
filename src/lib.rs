//! Loop Station Core Library
//!
//! Provides all audio processing, synchronization, and effect capabilities
//! for a professional-grade loop station application.

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod core {
    //! Core audio engine and track management
    pub mod engine;
    pub mod track;
    pub mod buffer;
}

pub mod audio {
    //! Audio processing modules
    pub mod effects;
    pub mod analysis;
    pub mod io {
        //! Audio input/output backends
        pub mod jack;
        pub mod file;
    }
}

pub mod sync {
    //! Synchronization and timing
    pub mod clock;
    pub mod quantize;
}

pub mod error {
    //! Error handling and logging
    pub mod types;
    pub mod logger;
}

pub mod state {
    //! Application state management
    pub mod config;
    pub mod preset;
    pub mod project;
}

/// Re-exports of commonly used types
pub mod prelude {
    pub use crate::{
        core::engine::AudioEngine,
        audio::effects::EffectsProcessor,
        sync::clock::MasterClock,
        sync::clock::Quantizer,
        error::types::{AudioError, AudioError::JackError, AudioError::TrackError},
        DEFAULT_SAMPLE_RATE
    };
}

/// Sample rate used throughout the application
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Main application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Number of audio input channels
    pub input_channels: usize,
    /// Number of audio output channels
    pub output_channels: usize,
    /// Initial BPM
    pub initial_bpm: f32,
    /// Maximum number of tracks
    pub max_tracks: usize,
    /// JACK client name
    pub client_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input_channels: 2,
            output_channels: 2,
            initial_bpm: 120.0,
            max_tracks: 8,
            client_name: "loop_station".into(),
        }
    }
}