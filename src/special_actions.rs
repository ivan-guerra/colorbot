//! Game-specific automation sequences and recovery actions.
//!
//! This module implements specialized bot behaviors such as inventory management,
//! obstacle course recovery sequences, and world hopping for specific game scenarios.

use crate::controls;

use anyhow::{Context, Result};

/// Switches to the next world using hotkeys.
#[allow(dead_code)]
pub fn hop_world() -> Result<()> {
    let world_hop_delay_sec = std::time::Duration::from_secs(10);

    controls::toggle_key("ctrl+shift+Right").context("Failed to press world hop hotkey")?;
    std::thread::sleep(world_hop_delay_sec);
    controls::toggle_key("Escape").context("Failed to press Escape for world hop")?;

    Ok(())
}
