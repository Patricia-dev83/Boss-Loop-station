//! Error types for the application

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Track error: {0}")]
    TrackError(String),
    
    #[error("JACK error: {0}")]
    JackError(#[from] jack::Error),
    
    #[error("Effect processing error: {0}")]
    EffectError(String),
    
    #[error("Buffer error: {0}")]
    BufferError(String),
    
    #[error("Channel mismatch")]
    ChannelMismatch,

    #[error("Port registration error: {0}")]
    PortRegistration(String), // Add this variant

    #[error("Activation error: {0}")]
    Activation(String),

    #[error("Invalid buffer error")]
    InvalidBuffer,

    #[error("Buffer mismatch error")]
    BufferMismatch, 

    #[error("Invalid state transition")]
    InvalidStateTransition,

    #[error("No state to undo")]
    NothingToUndo,

    #[error("No state to redo")]
    NothingToRedo,
    
}

// Add more error types as needed