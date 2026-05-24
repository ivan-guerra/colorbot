//! Colorbot: A scriptable Old School Runescape automation bot.
//!
//! This bot reads JSON event scripts and executes them in a loop for a specified duration,
//! supporting mouse movements, keypresses, color-based pixel detection, and custom actions.
use crate::config::BotConfig;
use crate::event::BotEvent;

use anyhow::{Context, Result};
use clap::Parser;
use log::debug;
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

mod config;
mod controls;
mod delay;
mod event;
mod vision;
mod windmouse;

/// Reads and parses a bot script from a JSON file.
fn read_bot_script(path: &Path) -> Result<Vec<BotEvent>> {
    let file = File::open(path).context("Failed to open bot script")?;
    let reader = BufReader::new(file);
    let events: Vec<BotEvent> =
        serde_json::from_reader(reader).context("Failed to parse bot script")?;

    Ok(events)
}

/// Executes the bot event loop repeatedly until the specified runtime expires.
fn run_event_loop(config: BotConfig) -> Result<()> {
    let events = read_bot_script(&config.script)?;
    debug!("Loaded {} events from script", events.len());

    let runtime = Duration::from_secs(config.runtime);
    let start_time = Instant::now();
    let end_time = start_time + runtime;
    debug!("Starting event loop for {} seconds", config.runtime);

    let mut iteration = 0;
    while Instant::now() < end_time {
        debug!("Starting iteration {}", iteration);

        for event in &events {
            event.exec(&config)?;
        }
        iteration += 1;
    }

    debug!("Event loop completed after {} iterations", iteration);
    Ok(())
}

/// Entry point that parses arguments, initializes logging, and runs the bot.
fn main() -> Result<()> {
    let config = BotConfig::parse();

    if config.debug {
        simplelog::TermLogger::init(
            simplelog::LevelFilter::Debug,
            simplelog::ConfigBuilder::new()
                .add_filter_allow_str("colorbot")
                .build(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        )
        .context("Failed to initialize logger")?;
    }

    run_event_loop(config).context("Failed to run event loop")?;

    Ok(())
}
