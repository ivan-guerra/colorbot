use crate::windmouse::{Point, WindMouse};

use anyhow::{Context, Result};
use device_query::{DeviceQuery, DeviceState};
use log::debug;
use std::{process::Command, time::Duration};

fn run_xdotool(args: &[&str]) -> Result<()> {
    Command::new("xdotool")
        .args(args)
        .output()
        .context(format!("Failed to execute xdotool with args: {:?}", args))?;
    Ok(())
}

fn get_mouse_pos() -> Point {
    let device_state = DeviceState::new();
    let mouse_state = device_state.get_mouse();

    Point::new(mouse_state.coords.0, mouse_state.coords.1)
}

pub fn move_mouse(target: Point) -> Result<()> {
    const MOUSE_SETTLE_DELAY_RNG_MS: std::ops::RangeInclusive<u64> = 50..=150;

    let start_pos = get_mouse_pos();
    let mut wind_mouse = WindMouse::new().context("failed to construct wind mouse object")?;

    debug!("Moving mouse from {} to {}", start_pos, target);
    wind_mouse
        .move_to(start_pos, target)
        .context("mouse move failed")?;

    std::thread::sleep(Duration::from_millis(rand::random_range(
        MOUSE_SETTLE_DELAY_RNG_MS,
    )));

    Ok(())
}

pub fn left_click() -> Result<()> {
    run_xdotool(&["click", "1"]).context("Failed to execute xdotool for left click")?;
    Ok(())
}

pub fn press_key(keycode: &str) -> Result<()> {
    const KEY_DELAY_RNG_MS: std::ops::RangeInclusive<u64> = 100..=150;

    run_xdotool(&["key", keycode])
        .context(format!("Failed to execute xdotool for key '{}'", keycode))?;

    std::thread::sleep(Duration::from_millis(rand::random_range(KEY_DELAY_RNG_MS)));

    Ok(())
}
