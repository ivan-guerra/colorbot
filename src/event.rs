use crate::geometry::PixelColor;
use crate::special_actions;
use crate::windmouse::Point;
use crate::{controls, geometry};

use anyhow::{bail, Context, Result};
use log::debug;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BotEvent {
    #[serde(rename = "mouse")]
    Mouse {
        id: String,
        pos: [u32; 2],
        delay_rng: [u32; 2],
    },
    #[serde(rename = "keypress")]
    KeyPress {
        id: String,
        keycode: String,
        delay_rng: [u32; 2],
        count: u32,
    },
    #[serde(rename = "color")]
    Color {
        id: String,
        rgb: [u8; 3],
        delay_rng: [u32; 2],
    },
    #[serde(rename = "special")]
    SpecialAction { id: String },
}

impl BotEvent {
    pub fn exec(&self) -> Result<()> {
        let sleep_random_delay = |delay_rng: &[u32; 2]| {
            let delay = rand::random_range(delay_rng[0]..=delay_rng[1]);
            debug!("Sleeping for {} ms", delay);
            std::thread::sleep(Duration::from_millis(delay.into()));
        };

        match &self {
            BotEvent::Mouse { id, pos, delay_rng } => {
                debug!("Executing mouse event '{}' at ({}, {})", id, pos[0], pos[1]);
                let point = Point::new(i32::try_from(pos[0])?, i32::try_from(pos[1])?);

                controls::move_mouse_with_rand(point)?;
                controls::left_click()?;
                sleep_random_delay(delay_rng);
            }
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
                let target_pixel = geometry::find_point_in_shape(&target_color)
                    .context("Failed to find target pixel color")?;

                controls::move_mouse(target_pixel)?;
                controls::left_click()?;
                sleep_random_delay(delay_rng);
            }
            BotEvent::SpecialAction { id } => {
                debug!("Executing special action '{}'", id);
                if id == "drop_inventory" {
                    special_actions::drop_inventory()
                        .context("Failed to execute inventory drop action")?;
                } else if id == "canifis_recovery" {
                    special_actions::canifis_recovery()
                        .context("Failed to execute Canifis recovery action")?;
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
