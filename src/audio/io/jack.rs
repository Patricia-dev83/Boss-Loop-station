//! JACK audio backend implementation
//!
//! Provides low-latency audio I/O using the JACK audio server with features:
//! - Multi-channel input/output
//! - Automatic client activation
//! - Sample-accurate timing
//! - Error recovery

use crate::{
    core::engine::AudioEngine,
    error::{AudioError, JackError},
};
use jack::{
    AsyncClient, Client, ClientOptions, Control, 
    Port, AudioIn, AudioOut, PortFlags, PortSpec,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tracing::{info, warn, error};

/// JACK audio client wrapper
pub struct JackAudio {
    client: AsyncClient<(), AudioEngine>,
    input_ports: Vec<Port<AudioIn>>,
    output_ports: Vec<Port<AudioOut>>,
    sample_rate: u32,
    active: Arc<AtomicBool>,
}

impl JackAudio {
    /// Create new JACK client and activate it
    pub fn new(
        engine: AudioEngine,
        client_name: &str,
        input_channels: usize,
        output_channels: usize,
    ) -> Result<Self, AudioError> {
        // Create JACK client
        let (client, status) = Client::new(
            client_name,
            ClientOptions::NO_START_SERVER,
        )
        .map_err(|e| JackError::ClientCreation(e.to_string()))?;

        // Log JACK status flags
        if !status.is_empty() {
            info!(
                "JACK client created with status: {:?}",
                status
            );
        }

        // Register ports
        let input_ports = (0..input_channels)
            .map(|i| {
                client.register_port(
                    &format!("input_{}", i + 1),
                    AudioIn::default(),
                )
                .map_err(|e| {
                    JackError::PortRegistration(format!(
                        "Input port {}: {}",
                        i + 1,
                        e
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let output_ports = (0..output_channels)
            .map(|i| {
                client.register_port(
                    &format!("output_{}", i + 1),
                    AudioOut::default(),
                )
                .map_err(|e| {
                    JackError::PortRegistration(format!(
                        "Output port {}: {}",
                        i + 1,
                        e
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sample_rate = client.sample_rate();
        let active = Arc::new(AtomicBool::new(true));

        // Activate client
        let active_clone = active.clone();
        let async_client = client
            .activate_async(
                (),
                move |client: &Client, ps: &jack::ProcessScope| {
                    Self::process_callback(
                        client,
                        ps,
                        &engine,
                        &active_clone,
                    )
                },
            )
            .map_err(|e| JackError::Activation(e.to_string()))?;

        info!(
            "JACK client activated with {} inputs, {} outputs at {}Hz",
            input_channels,
            output_channels,
            sample_rate
        );

        Ok(Self {
            client: async_client,
            input_ports,
            output_ports,
            sample_rate,
            active,
        })
    }

    /// Main audio processing callback
    fn process_callback(
        client: &Client,
        ps: &jack::ProcessScope,
        engine: &AudioEngine,
        active: &AtomicBool,
    ) -> jack::Control {
        if !active.load(Ordering::SeqCst) {
            return Control::Quit;
        }

        // Get audio buffers
        let input_buffers: Vec<&[f32]> = client
            .ports(
                None,
                None,
                PortFlags::IS_INPUT | PortFlags::IS_PHYSICAL,
            )
            .unwrap_or_default()
            .iter()
            .filter_map(|name| {
                client
                    .port_by_name(name)
                    .and_then(|p| p.as_slice(ps))
            })
            .collect();

        let output_buffers: Vec<&mut [f32]> = client
            .ports(
                None,
                None,
                PortFlags::IS_OUTPUT | PortFlags::IS_PHYSICAL,
            )
            .unwrap_or_default()
            .iter()
            .filter_map(|name| {
                client
                    .port_by_name(name)
                    .and_then(|p| p.as_mut_slice(ps))
            })
            .collect();

        // Process audio through engine
        match engine.process(&input_buffers, &output_buffers) {
            Ok(_) => Control::Continue,
            Err(e) => {
                error!("Audio processing error: {}", e);
                active.store(false, Ordering::SeqCst);
                Control::Quit
            }
        }
    }

    /// Get current JACK sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Check if client is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Gracefully shutdown JACK client
    pub fn shutdown(&mut self) -> Result<(), AudioError> {
        self.active.store(false, Ordering::SeqCst);
        info!("JACK client shutdown initiated");
        Ok(())
    }

    /// Get JACK client latency information
    pub fn get_latency(&self) -> Result<Duration, AudioError> {
        let frames = self
            .client
            .as_client()
            .port_by_name(&format!("{}:output_1", self.client.name()))
            .and_then(|p| p.latency_range())
            .map(|(min, max)| max)
            .unwrap_or(0);

        Ok(Duration::from_secs_f64(
            frames as f64 / self.sample_rate as f64,
        ))
    }
}

impl Drop for JackAudio {
    fn drop(&mut self) {
        if self.is_active() {
            let _ = self.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::engine::AudioEngine;

    #[test]
    fn test_jack_initialization() {
        let engine = AudioEngine::new(44100).unwrap();
        let jack = JackAudio::new(engine, "test_client", 2, 2);
        
        assert!(jack.is_ok());
        if let Ok(jack) = jack {
            assert_eq!(jack.sample_rate(), 44100);
            assert!(jack.is_active());
            
            // Test shutdown
            assert!(jack.shutdown().is_ok());
            assert!(!jack.is_active());
        }
    }
}