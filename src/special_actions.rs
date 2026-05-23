//! Game-specific automation sequences and recovery actions.
//!
//! This module implements specialized bot behaviors such as inventory management,
//! obstacle course recovery sequences, and world hopping for specific game scenarios.

use crate::event::BotEvent;
use crate::vision::PixelColor;
use crate::{controls, vision};

use anyhow::{Context, Result};
use log::debug;

/// Switches to the next world using hotkeys.
#[allow(dead_code)]
fn hop_world() -> Result<()> {
    let world_hop_delay_sec = std::time::Duration::from_secs(10);

    controls::toggle_key("ctrl+shift+Right").context("Failed to press world hop hotkey")?;
    std::thread::sleep(world_hop_delay_sec);
    controls::toggle_key("Escape").context("Failed to press Escape for world hop")?;

    Ok(())
}

/// Clicks on the cave entrance and waits for entry animation.
fn enter_gemstone_cave() -> Result<()> {
    let click_cave = BotEvent::Color {
        id: "tile 1".to_string(),
        rgb: [0, 0, 255],
        delay_rng: [18000, 18500],
    };

    click_cave.exec()?;

    Ok(())
}

/// Continuously enters the cave until the gemstone crab is detected.
pub fn find_gemstone_crab() -> Result<()> {
    let magenta_pixel = PixelColor::new(255, 0, 255);

    loop {
        let detected_crab = vision::find_color(&magenta_pixel)?;

        if detected_crab.is_some() {
            break;
        } else {
            debug!("No gemstone crab detected, entering cave");
            enter_gemstone_cave()?;
        }
    }
    Ok(())
}
