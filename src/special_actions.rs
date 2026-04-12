use crate::event::BotEvent;
use crate::geometry::PixelColor;
use crate::{controls, geometry};

use anyhow::{Context, Result};
use log::debug;

pub fn drop_inventory() -> Result<()> {
    const INVENTORY_ROWS: usize = 7;
    const INVENTORY_COLS: usize = 4;

    let cyan_pixel = PixelColor::new(0, 255, 255);

    // Pass through twice to make sure we drop all items as the algorithm sometimes misses items on
    // the first pass
    for _ in 0..2 * INVENTORY_ROWS * INVENTORY_COLS {
        let inventory_pos = geometry::find_color(&cyan_pixel)
            .context("Failed to run drop inventory color check")?;

        if let Some(pos) = inventory_pos {
            controls::move_mouse(pos)?;
            controls::left_click()?;
        }
    }
    Ok(())
}

pub fn canifis_recovery() -> Result<()> {
    let red_pixel = PixelColor::new(255, 0, 0);
    let detected_failure =
        geometry::find_color(&red_pixel).context("Failed to run canifis recovery color check")?;

    if detected_failure.is_some() {
        debug!("Detected obstacle failure, executing Canifis recovery");

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
    } else {
        debug!("No obstacle failure detected, skipping Canifis recovery");
    }

    Ok(())
}

#[allow(dead_code)]
fn hop_world() -> Result<()> {
    let world_hop_delay_sec = std::time::Duration::from_secs(10);

    controls::toggle_key("ctrl+shift+Right").context("Failed to press world hop hotkey")?;
    std::thread::sleep(world_hop_delay_sec);
    controls::toggle_key("Escape").context("Failed to press Escape for world hop")?;

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
    let magenta_pixel = PixelColor::new(255, 0, 255);

    loop {
        let detected_crab = geometry::find_color(&magenta_pixel)?;

        if detected_crab.is_some() {
            break;
        } else {
            debug!("No gemstone crab detected, entering cave");
            enter_gemstone_cave()?;
        }
    }
    Ok(())
}
