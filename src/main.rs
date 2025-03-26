//! Loop Station - Professional Audio Looper
//!
//! Command-line interface for the loop station application.

use loop_station::{
    prelude::*,
    audio::io::jack::JackAudio,
    state::config::AppConfig,
    DEFAULT_SAMPLE_RATE,
};
use clap::Parser;
use std::{
    sync::Arc,
    time::Duration,
    thread,
};
use ctrlc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Number of input channels
    #[arg(short, long, default_value_t = 2)]
    inputs: usize,
    
    /// Number of output channels
    #[arg(short, long, default_value_t = 2)]
    outputs: usize,
    
    /// Initial BPM
    #[arg(long, default_value_t = 120.0)]
    bpm: f32,
    
    /// JACK client name
    #[arg(long, default_value = "loop_station")]
    client_name: String,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    let _logger = error::logger::ErrorLogger::new(log_level);
    
    // Create application configuration
    let config = AppConfig {
        input_channels: cli.inputs,
        output_channels: cli.outputs,
        initial_bpm: cli.bpm,
        client_name: cli.client_name,
        ..Default::default()
    };
    
    info!("Starting loop station with config: {:?}", config);
    
    // Initialize audio engine
    let engine = Arc::new(tokio::sync::Mutex::new(
        AudioEngine::new(DEFAULT_SAMPLE_RATE, config.max_tracks)?
    ));
    
    // Create JACK client
    let mut jack = JackAudio::new(
        engine.clone(),
        &config.client_name,
        config.input_channels,
        config.output_channels,
    )?;
    
    info!("Audio engine initialized at {}Hz", jack.sample_rate());
    
    // Set up CTRL+C handler
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Shutdown signal received");
    })?;
    
    // Main application loop
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
        
        // Here you would typically:
        // 1. Handle UI updates
        // 2. Process MIDI input
        // 3. Manage track state
    }
    
    // Graceful shutdown
    jack.shutdown()?;
    info!("Application shutdown complete");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.input_channels, 2);
        assert_eq!(config.initial_bpm, 120.0);
    }
}