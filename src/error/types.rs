//! Error types

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("JACK audio error: {0}")]
    JackError(#[from] jack::Error),
    
    #[error("Track operation error")]
    TrackError {
        track_id: usize,
        source: TrackError
    },
    // ... other variants
}