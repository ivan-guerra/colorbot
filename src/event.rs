//! Bot event types and execution logic.
//!
//! This module defines the core event types (keypresses, color detection, and image template
//! recognition) that can be deserialized from bot scripts and executed with randomized delays for
//! human-like automation.
use crate::config::BotConfig;
use crate::delay::DelayModel;
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

    /// Base delay in milliseconds before executing the event.
    pub delay: u64,

    /// The specific event type and its parameters.
    #[serde(flatten)]
    pub event_type: BotEventType,
}

fn default_count() -> u32 {
    1
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
    pub fn exec(&self, config: &BotConfig) -> Result<()> {
        // Sleeps for a randomized duration based on the configured delay model
        let sleep = |delay: u64| -> Result<()> {
            const GAMMA_SHAPE: f64 = 1.5; // Shape that's not too clustered around the mean,
                                          // allowing for more variability
            let scale_ms = config.added_delay as f64 / GAMMA_SHAPE;
            let max_delay_ms = Duration::from_millis(delay + config.max_added_delay);
            let model = DelayModel::new(Duration::from_millis(delay))
                .with_short_gamma(GAMMA_SHAPE, scale_ms)
                .with_max_delay(max_delay_ms);
            let mut rng = rand::rng();
            let random_delay = model.next_delay(&mut rng)?;

            debug!("Sleeping for {:?} before next action", random_delay);
            std::thread::sleep(random_delay);

            Ok(())
        };

        for i in 0..self.count {
            if self.count > 1 {
                debug!("Executing '{}' iteration {}/{}", self.id, i + 1, self.count);
            }

            match &self.event_type {
                BotEventType::KeyPress { keycode } => {
                    debug!("Executing keypress '{}': '{}'", self.id, keycode);
                    controls::toggle_key(keycode)?;
                    sleep(self.delay)?;
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
                    sleep(self.delay)?;
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
                    sleep(self.delay)?;
                }
            }
        }
        Ok(())
    }
}
