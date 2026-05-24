use clap::Parser;

/// Command-line configuration for the bot runtime and script.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct BotConfig {
    /// Path to the JSON bot script file containing event sequences.
    pub script: std::path::PathBuf,

    /// Duration in seconds for the bot to run before stopping.
    #[arg(short = 'r', long, default_value_t = 3_600)]
    pub runtime: u64,

    /// Enable debug logging output to terminal.
    #[arg(short = 'g', long, default_value_t = false)]
    pub debug: bool,

    // Average additional delay in ms to add to each script event delay.
    #[arg(short = 'd', long, default_value_t = 500)]
    pub added_delay: u64,

    /// Maximum additional delay in ms that can be added to each script event delay.
    #[arg(short = 'm', long, default_value_t = 1_000)]
    pub max_added_delay: u64,
}
