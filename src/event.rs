//! Bot event types and execution logic.
//!
//! This module defines the core event types (keypresses, color detection, and image template
//! recognition) that can be deserialized from bot scripts and executed with randomized delays for
//! human-like automation.
use crate::vision::PixelColor;
use crate::{controls, vision};

use anyhow::{Context, Result};
use log::debug;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Represents different types of bot events that can be executed.
#[derive(Debug, Deserialize)]
pub struct BotEvent {
    /// Event identifier for logging.
    pub id: String,
    /// Number of times to execute this event.
    #[serde(default = "default_count")]
    pub count: u32,
    /// Delay range in milliseconds [min, max] after execution.
    #[serde(default = "default_delay_rng")]
    pub delay_rng: [u32; 2],
    /// The specific event type and its parameters.
    #[serde(flatten)]
    pub event_type: BotEventType,
}

fn default_count() -> u32 {
    1
}

fn default_delay_rng() -> [u32; 2] {
    [0, 0]
}

/// The specific type of bot event.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BotEventType {
    /// Keyboard key press event.
    #[serde(rename = "keypress")]
    KeyPress {
        /// Key to press (xdotool format).
        keycode: String,
    },
    /// Color-based pixel detection and click event.
    #[serde(rename = "color")]
    Color {
        /// Target RGB color values [r, g, b].
        rgb: [u8; 3],
    },
    #[serde(rename = "image")]
    Image {
        /// Path to the image file to search for on the screen.
        image_path: PathBuf,
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

        for i in 0..self.count {
            if self.count > 1 {
                debug!("Executing '{}' iteration {}/{}", self.id, i + 1, self.count);
            }

            match &self.event_type {
                BotEventType::KeyPress { keycode } => {
                    debug!("Executing keypress '{}': '{}'", self.id, keycode);
                    controls::toggle_key(keycode)?;
                    sleep_random_delay(&self.delay_rng);
                }
                BotEventType::Color { rgb } => {
                    debug!(
                        "Executing color event '{}': target RGB({},{},{})",
                        self.id, rgb[0], rgb[1], rgb[2]
                    );
                    let target_color = PixelColor::new(rgb[0], rgb[1], rgb[2]);
                    let target_pixel = vision::find_point_in_shape(&target_color)
                        .context("Failed to find target pixel color")?;

                    controls::move_mouse(target_pixel)?;
                    controls::left_click()?;
                    sleep_random_delay(&self.delay_rng);
                }
                BotEventType::Image { image_path } => {
                    debug!(
                        "Executing image event '{}': searching for image '{}'",
                        self.id,
                        image_path.display()
                    );
                    let target_pixel = vision::find_image_on_screen(image_path)
                        .context("Failed to find target image on screen")?;
                    controls::move_mouse(target_pixel)?;
                    controls::left_click()?;
                    sleep_random_delay(&self.delay_rng);
                }
            }
        }
        Ok(())
    }
}
