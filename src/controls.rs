use crate::windmouse::{Point, WindMouse};
use anyhow::{Context, Result};
use device_query::{DeviceQuery, DeviceState};
use log::debug;
use scrap::{Capturer, Display};
use std::{process::Command, time::Duration};

fn run_xdotool(args: &[&str]) -> Result<()> {
    Command::new("xdotool")
        .args(args)
        .output()
        .context(format!("Failed to execute xdotool with args: {:?}", args))?;
    Ok(())
}

#[derive(Debug)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn is_match(&self, target: &PixelColor, tolerance: u8) -> bool {
        (i16::from(self.r) - i16::from(target.r)).abs() <= i16::from(tolerance)
            && (i16::from(self.g) - i16::from(target.g)).abs() <= i16::from(tolerance)
            && (i16::from(self.b) - i16::from(target.b)).abs() <= i16::from(tolerance)
    }
}

pub fn get_pixels_with_target_color(target_color: &PixelColor) -> Result<Vec<Point>> {
    // Get the primary display
    let display = Display::primary()?;
    let width = display.width();
    let mut capturer = Capturer::new(display).context("Failed to create Capturer object")?;
    let mut matches = Vec::new();
    const TOLERANCE: u8 = 3;

    loop {
        // Try to capture a frame
        if let Ok(frame) = capturer.frame() {
            // Iterate over the pixels
            for (i, bgra) in frame.chunks(4).enumerate() {
                let curr_color = PixelColor::new(bgra[2], bgra[1], bgra[0]);

                if curr_color.is_match(target_color, TOLERANCE) {
                    // Calculate pixel coordinates
                    let x = i % width;
                    let y = i / width;
                    matches.push(Point::new(i32::try_from(x)?, i32::try_from(y)?));
                }
            }
            break; // Exit after one frame
        }
    }
    Ok(matches)
}

pub fn calculate_centroid(boundary_points: &[Point]) -> Point {
    // Clone and work on the points
    let mut points = boundary_points.to_vec();

    // Step 1: Find the geometric center
    let center_x = points.iter().map(|p| p.x).sum::<i32>() / points.len() as i32;
    let center_y = points.iter().map(|p| p.y).sum::<i32>() / points.len() as i32;
    let center = Point::new(center_x, center_y);

    // Step 2: Sort points counterclockwise around the center
    points.sort_by(|p1, p2| {
        let angle1 = f64::from(p1.y - center.y).atan2(f64::from(p1.x - center.x));
        let angle2 = f64::from(p2.y - center.y).atan2(f64::from(p2.x - center.x));
        angle1.partial_cmp(&angle2).unwrap()
    });

    // Step 3: Ensure the shape is closed
    if points.first() != points.last() {
        points.push(points[0]);
    }

    // Step 4: Calculate the centroid using the Shoelace formula
    let mut area = 0;
    let mut cx = 0;
    let mut cy = 0;

    for i in 0..points.len() - 1 {
        let p1 = points[i];
        let p2 = points[i + 1];
        let cross = p1.x * p2.y - p2.x * p1.y;
        area += cross;
        cx += (p1.x + p2.x) * cross;
        cy += (p1.y + p2.y) * cross;
    }

    area /= 2;
    cx /= 6 * area;
    cy /= 6 * area;

    Point::new(cx, cy)
}

fn get_mouse_pos() -> Point {
    let device_state = DeviceState::new();
    let mouse_state = device_state.get_mouse();

    Point::new(mouse_state.coords.0, mouse_state.coords.1)
}

pub fn move_mouse(target: Point) -> Result<()> {
    const MOUSE_SETTLE_DELAY_RNG_MS: std::ops::RangeInclusive<u64> = 50..=150;

    let start_pos = get_mouse_pos();
    let target_rand = Point::new(
        target.x + rand::random_range(-5..=5),
        target.y + rand::random_range(-5..=5),
    );
    let mut wind_mouse = WindMouse::new().context("failed to construct wind mouse object")?;

    debug!("Moving mouse from {} to {}", start_pos, target_rand);
    wind_mouse
        .move_to(start_pos, target_rand)
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
