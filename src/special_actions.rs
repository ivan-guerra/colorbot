use crate::controls;
use crate::event::BotEvent;

use crate::windmouse::Point;
use anyhow::{Context, Result};
use log::debug;

pub fn drop_inventory() -> Result<()> {
    const INVENTORY_ROWS: usize = 7;
    const INVENTORY_COLS: usize = 4;
    const BASE_X: usize = 743;
    const BASE_Y: usize = 754;
    const COL_SPACING: usize = 40;
    const ROW_SPACING: usize = 37;

    for row in 0..INVENTORY_ROWS {
        for col in 0..INVENTORY_COLS {
            let x = BASE_X + col * COL_SPACING;
            let y = BASE_Y + row * ROW_SPACING;
            let inventory_pos = Point::new(i32::try_from(x)?, i32::try_from(y)?);

            controls::move_mouse(inventory_pos)?;
            controls::left_click()?;
        }
    }
    Ok(())
}

pub fn canifis_recovery() -> Result<()> {
    let red_bgra = (0, 0, 255, 0);
    let matches = controls::get_pixels_with_target_color(&red_bgra)?;
    if matches.is_empty() {
        debug!("No obstacle failure detected, skipping Canifis recovery");
    } else {
        debug!(
            "Detected obstacle failure with {} matching pixels, executing Canifis recovery",
            matches.len()
        );
        let canifis_recovery_events = vec![
            BotEvent::Color {
                id: "tile 1".to_string(),
                rgb: [255, 0, 255],
                delay_rng: [4500, 4750],
            },
            BotEvent::Color {
                id: "tile 2".to_string(),
                rgb: [0, 0, 255],
                delay_rng: [6500, 6750],
            },
            BotEvent::Color {
                id: "tile 3".to_string(),
                rgb: [255, 0, 0],
                delay_rng: [5500, 5750],
            },
            BotEvent::Mouse {
                id: "obstacle 1".to_string(),
                pos: [398, 399],
                delay_rng: [7500, 7750],
            },
            BotEvent::Color {
                id: "mark of grace".to_string(),
                rgb: [0, 255, 255],
                delay_rng: [5000, 5250],
            },
            BotEvent::Color {
                id: "obstacle 2".to_string(),
                rgb: [255, 0, 0],
                delay_rng: [5500, 5750],
            },
            BotEvent::Color {
                id: "mark of grace".to_string(),
                rgb: [0, 255, 255],
                delay_rng: [5000, 5250],
            },
            BotEvent::Color {
                id: "obstacle 3".to_string(),
                rgb: [0, 0, 255],
                delay_rng: [4500, 4750],
            },
            BotEvent::Color {
                id: "mark of grace".to_string(),
                rgb: [0, 255, 255],
                delay_rng: [5000, 5250],
            },
            BotEvent::Color {
                id: "obstacle 4".to_string(),
                rgb: [255, 0, 255],
                delay_rng: [6000, 6250],
            },
            BotEvent::SpecialAction {
                id: "canifis_recovery".to_string(),
            },
        ];

        for event in canifis_recovery_events {
            event.exec()?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn hop_world() -> Result<()> {
    let world_hop_delay_sec = std::time::Duration::from_secs(10);

    controls::press_key("ctrl+shift+Right").context("Failed to press world hop hotkey")?;
    std::thread::sleep(world_hop_delay_sec);
    controls::press_key("Escape").context("Failed to press Escape for world hop")?;

    Ok(())
}

fn enter_gemstone_cave() -> Result<()> {
    let click_cave = BotEvent::Color {
        id: "tile 1".to_string(),
        rgb: [0, 0, 255],
        delay_rng: [18000, 18500],
    };

    click_cave.exec()?;

    Ok(())
}

pub fn find_gemstone_crab() -> Result<()> {
    let magenta_bgra = (255, 0, 255, 0);

    loop {
        let matches = controls::get_pixels_with_target_color(&magenta_bgra)?;

        if matches.is_empty() {
            debug!("No gemstone crab detected, entering cave");
            enter_gemstone_cave()?;
        } else {
            break;
        }
    }
    Ok(())
}
