//! Bot event types and execution logic.
//!
//! This module defines the core event types (mouse clicks, keypresses, color detection, and special actions)
//! that can be deserialized from bot scripts and executed with randomized delays for human-like automation.
use crate::special_actions;
use crate::vision::PixelColor;
use crate::{controls, vision};

use anyhow::{bail, Context, Result};
use log::debug;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Represents different types of bot events that can be executed.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BotEvent {
    /// Keyboard key press event.
    #[serde(rename = "keypress")]
    KeyPress {
        /// Event identifier for logging.
        id: String,
        /// Key to press (xdotool format).
        keycode: String,
        /// Delay range in milliseconds [min, max] after execution.
        delay_rng: [u32; 2],
        /// Number of times to press the key.
        count: u32,
    },
    /// Color-based pixel detection and click event.
    #[serde(rename = "color")]
    Color {
        /// Event identifier for logging.
        id: String,
        /// Target RGB color values [r, g, b].
        rgb: [u8; 3],
        /// Delay range in milliseconds [min, max] after execution.
        delay_rng: [u32; 2],
    },
    #[serde(rename = "image")]
    Image {
        /// Event identifier for logging.
        id: String,
        /// Path to the image file to search for on the screen.
        image_path: PathBuf,
        /// Delay range in milliseconds [min, max] after execution.
        delay_rng: [u32; 2],
    },
    /// Special predefined action sequence.
    #[serde(rename = "special")]
    SpecialAction {
        /// Action identifier (e.g., "drop_inventory", "canifis_recovery").
        id: String,
    },
}

impl BotEvent {
    /// Executes the bot event based on its type.
    pub fn exec(&self) -> Result<()> {
        // Sleeps for a random duration within the specified range.
        let sleep_random_delay = |delay_rng: &[u32; 2]| {
            let delay = rand::random_range(delay_rng[0]..=delay_rng[1]);
            debug!("Sleeping for {} ms", delay);
            std::thread::sleep(Duration::from_millis(delay.into()));
        };

        match &self {
            BotEvent::KeyPress {
                id,
                keycode,
                delay_rng,
                count,
            } => {
                debug!("Executing keypress '{}': '{}' x{}", id, keycode, count);

                for _ in 0..*count {
                    controls::toggle_key(keycode)?;
                }
                sleep_random_delay(delay_rng);
            }
            BotEvent::Color { id, rgb, delay_rng } => {
                debug!(
                    "Executing color event '{}': target RGB({},{},{})",
                    id, rgb[0], rgb[1], rgb[2]
                );
                let target_color = PixelColor::new(rgb[0], rgb[1], rgb[2]);
                let target_pixel = vision::find_point_in_shape(&target_color)
                    .context("Failed to find target pixel color")?;

                controls::move_mouse(target_pixel)?;
                controls::left_click()?;
                sleep_random_delay(delay_rng);
            }
            BotEvent::Image {
                id,
                image_path,
                delay_rng,
            } => {
                debug!(
                    "Executing image event '{}': searching for image '{}'",
                    id,
                    image_path.display()
                );
                let target_pixel = vision::find_image_on_screen(image_path)
                    .context("Failed to find target image on screen")?;
                controls::move_mouse(target_pixel)?;
                controls::left_click()?;
                sleep_random_delay(delay_rng);
            }
            BotEvent::SpecialAction { id } => {
                debug!("Executing special action '{}'", id);
                if id == "drop_inventory" {
                    special_actions::drop_inventory()
                        .context("Failed to execute inventory drop action")?;
                } else if id == "find_crab" {
                    special_actions::find_gemstone_crab()
                        .context("Failed to execute find gemstone crab action")?;
                } else {
                    bail!("Unknown special action '{}'", id);
                }
            }
        }
        Ok(())
    }
}
