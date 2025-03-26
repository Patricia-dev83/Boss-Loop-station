//! JACK audio backend implementation
//!
//! Provides low-latency audio I/O using the JACK audio server with features:
//! - Multi-channel input/output
//! - Automatic client activation
//! - Sample-accurate timing
//! - Error recovery

//! JACK audio backend implementation

use crate::{
    core::engine::AudioEngine,
    prelude::{AudioError, JackError},
};
use jack::{
    AsyncClient, Client, ClientOptions, Control,
    AudioIn, AudioOut, PortFlags,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tracing::{info, error};

pub struct JackAudio {
    client: AsyncClient<(), ProcessHandler>,
    sample_rate: u32,
    active: Arc<AtomicBool>,
}

struct ProcessHandler {
    engine: AudioEngine,
    active: Arc<AtomicBool>,
}

impl JackAudio {
    pub fn new(
        engine: AudioEngine,
        client_name: &str,
        input_channels: usize,
        output_channels: usize,
    ) -> Result<Self, AudioError> {
        let (client, status) = Client::new(
            client_name,
            ClientOptions::NO_START_SERVER,
        )
        .map_err(|e| AudioError::JackError(jack::Error::from(e)))?;

        if !status.is_empty() {
            info!("JACK client status: {:?}", status);
        }

        // Register input ports
        for i in 0..input_channels {
            client.register_port(
                &format!("input_{}", i + 1),
                AudioIn::default(),
            )
            .map_err(|e| AudioError::PortRegistration(format!("Input port {}: {}", i + 1, e)))?;
        }

        // Register output ports
        for i in 0..output_channels {
            client.register_port(
                &format!("output_{}", i + 1),
                AudioOut::default(),
            )
            .map_err(|e| AudioError::PortRegistration(format!("Output port {}: {}", i + 1, e)))?;
        }

        let sample_rate = client.sample_rate();
        let active = Arc::new(AtomicBool::new(true));

        let process_handler = ProcessHandler {
            engine,
            active: active.clone(),
        };

        let async_client = client
            .activate_async((), process_handler)
            .map_err(|e| AudioError::Activation(e.to_string()))?;

        info!(
            "JACK client activated with {} inputs, {} outputs at {}Hz",
            input_channels, output_channels, sample_rate
        );

        Ok(Self {
            client: async_client,
            sample_rate: sample_rate.try_into().unwrap(),
            active,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn shutdown(&mut self) -> Result<(), AudioError> {
        self.active.store(false, Ordering::SeqCst);
        info!("JACK client shutdown");
        Ok(())
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_latency(&self) -> Result<Duration, AudioError> {
        let port_name = format!("{}:output_1", self.client.as_client().name());
        let frames = self.client.as_client()
            .port_by_name(&port_name)
            .map_or(0, |port| {
                let (_, max) = port.get_latency_range(jack::LatencyType::Playback);
                max
            });

        Ok(Duration::from_secs_f64(frames as f64 / self.sample_rate as f64))
    }
}

impl jack::ProcessHandler for ProcessHandler {
    fn process(&mut self, client: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        if !self.active.load(Ordering::SeqCst) {
            return Control::Quit;
        }

        // Get all input buffers
        let input_buffers: Vec<&[f32]> = client
            .ports(None, None, PortFlags::IS_INPUT | PortFlags::IS_PHYSICAL)
            .iter()
            .filter_map(|name| {
                client.port_by_name(name)
                    .and_then(|port| {
                        // Use buffer() instead of audio_buffer()
                        port.buffer(ps).ok()
                    })
            })
            .collect();

        // Get all output buffers
        let mut output_buffers: Vec<&mut [f32]> = client
            .ports(None, None, PortFlags::IS_OUTPUT | PortFlags::IS_PHYSICAL)
            .iter()
            .filter_map(|name| {
                client.port_by_name(name)
                    .and_then(|port| {
                        // Use buffer() instead of audio_buffer()
                        port.buffer(ps).ok()
                    })
            })
            .collect();

        match self.engine.process(&input_buffers, &mut output_buffers) {
            Ok(_) => Control::Continue,
            Err(e) => {
                error!("Processing error: {}", e);
                self.active.store(false, Ordering::SeqCst);
                Control::Quit
            }
        }
    }
}

impl Drop for JackAudio {
    fn drop(&mut self) {
        if self.is_active() {
            let _ = self.shutdown();
        }
    }
}