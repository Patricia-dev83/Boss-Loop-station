//! MIDI event handler

// src/midi/handler.rs
pub struct MidiHandler {
    input: midir::MidiInput,
    connections: Vec<midir::MidiInputConnection<()>>,
}

impl MidiHandler {
    pub fn new() -> Result<Self> {
        let input = midir::MidiInput::new("loop_station_midi")?;
        Ok(Self {
            input,
            connections: Vec::new(),
        })
    }

    pub fn connect<F>(&mut self, port_index: usize, callback: F) -> Result<()>
    where
        F: FnMut(u64, &[u8], &mut ()) + Send + 'static
    {
        let conn = self.input.connect(
            port_index,
            "midi_in",
            callback,
            (),
        )?;
        self.connections.push(conn);
        Ok(())
    }
}