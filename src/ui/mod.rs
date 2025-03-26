//! User interface implementations
pub mod cli;
pub mod tui;

// src/ui/tui/mod.rs
pub fn run_tui(engine: Arc<Mutex<AudioEngine>>) -> Result<()> {
    // Terminal initialization
    let mut terminal = setup_terminal()?;
    
    // Main loop
    while running.load(Ordering::Relaxed) {
        terminal.draw(|f| {
            // Render tracks
            let tracks = engine.lock().unwrap().tracks();
            render_tracks(f, tracks);
            
            // Render transport controls
            render_transport(f);
        })?;
        
        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => running.store(false, Ordering::Relaxed),
                // Other controls
            }
        }
    }
    
    restore_terminal()?;
    Ok(())
}