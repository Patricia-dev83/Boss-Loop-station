//! CLI argument parser

// src/ui/cli/parser.rs
#[derive(clap::Subcommand)]
pub enum Command {
    /// Record audio
    Record {
        #[arg(short, long)]
        track: usize,
        #[arg(short, long)]
        length: Option<f32>,
    },
    /// Apply effect to track
    Effect {
        #[arg(short, long)]
        track: usize,
        #[arg(subcommand)]
        effect: EffectCommand,
    },
}

pub fn parse() -> Result<AppState> {
    let cli = Cli::parse();
    match cli.command {
        Command::Record { track, length } => {
            // Handle recording
        }
        Command::Effect { track, effect } => {
            // Handle effects
        }
    }
}