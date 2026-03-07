use crate::controls::{get_pixels_with_target_color, left_click, move_mouse};
use crate::event::BotEvent;

use anyhow::Result;
use kurbo::Point;
use log::debug;

pub fn drop_inventory() -> Result<()> {
    const INVENTORY_ROWS: usize = 7;
    const INVENTORY_COLS: usize = 4;
    const BASE_X: f64 = 743.0;
    const BASE_Y: f64 = 754.0;
    const COL_SPACING: f64 = 40.0;
    const ROW_SPACING: f64 = 37.0;

    for row in 0..INVENTORY_ROWS {
        for col in 0..INVENTORY_COLS {
            let x = BASE_X + col as f64 * COL_SPACING;
            let y = BASE_Y + row as f64 * ROW_SPACING;
            let inventory_pos = Point::new(x, y);

            move_mouse(inventory_pos)?;
            left_click()?;
        }
    }
    Ok(())
}

pub fn canifis_recovery() -> Result<()> {
    let red_bgra = (0, 0, 255, 0);
    let matches = get_pixels_with_target_color(&red_bgra)?;
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

pub fn logout() -> Result<()> {
    let logout_events = vec![
        BotEvent::Mouse {
            id: "logout door".to_string(),
            pos: [806, 1011],
            delay_rng: [3000, 3001],
        },
        BotEvent::Mouse {
            id: "click here to logout".to_string(),
            pos: [803, 958],
            delay_rng: [3000, 3001],
        },
    ];

    for event in logout_events {
        event.exec()?;
    }
    Ok(())
}
