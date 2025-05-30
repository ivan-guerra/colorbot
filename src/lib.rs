//! A color-based automation bot that simulates mouse movements and clicks
//!
//! This module provides functionality to:
//! - Read and execute bot scripts defined in JSON format
//! - Detect pixels of specific colors on the screen
//! - Generate natural-looking mouse movements using Bézier curves
//! - Simulate mouse clicks using xdotool on Linux systems
use device_query::{DeviceQuery, DeviceState};
use kurbo::{CubicBez, ParamCurve, Point};
use log::{debug, warn};
use rand::Rng;
use scrap::{Capturer, Display};
use serde::Deserialize;
use std::io::Write;

/// Configuration struct for the bot
pub struct BotConfig {
    /// Path to the script file to execute
    pub script: std::path::PathBuf,
    /// Runtime duration in seconds
    pub runtime: u32,
    /// Maximum deviation for mouse movements in pixels
    pub mouse_deviation: u32,
    /// Mouse movement speed in pixels per second
    pub mouse_speed: u32,
    /// Enable debug logging
    pub debug: bool,
}

impl BotConfig {
    pub fn new(
        script: std::path::PathBuf,
        runtime: u32,
        mouse_deviation: u32,
        mouse_speed: u32,
        debug: bool,
    ) -> Self {
        Self {
            script,
            runtime,
            mouse_deviation,
            mouse_speed,
            debug,
        }
    }
}

/// Represents a mouse event with timing and color information
#[derive(Deserialize, Debug, Clone)]
pub struct MouseEvent {
    /// Unique identifier for the mouse event
    pub id: String,
    /// RGB color values as an array of 3 bytes
    pub color: [u8; 3],
    pub position: [u32; 2],
    /// Range for random delay timing [min, max] in milliseconds
    pub delay_rng: [u32; 2],
}

/// Represents a collection of mouse events forming a bot script
#[derive(Deserialize, Debug)]
struct BotScript {
    /// Vector of mouse events to be executed in sequence
    events: Vec<MouseEvent>,
}

/// Generates a cubic Bézier curve to simulate natural mouse movement between two points
///
/// # Arguments
///
/// * `init_pos` - The starting point of the mouse movement
/// * `fin_pos` - The ending point of the mouse movement
/// * `deviation` - The maximum percentage of deviation from a straight line (0-100)
///
/// # Returns
///
/// A `CubicBez` curve representing the mouse movement path
pub fn mouse_bez(init_pos: Point, fin_pos: Point, deviation: u32) -> CubicBez {
    let get_ctrl_point = |init_pos: f64, fin_pos: f64| {
        let mut rng = rand::thread_rng();
        let choice = if rng.gen_bool(0.5) { 1 } else { -1 };
        let dev = rng.gen_range(deviation / 2..=deviation) as i32;
        let diff = (fin_pos as i32).saturating_sub(init_pos as i32);
        let offset = (choice * diff * dev) / 100;
        (init_pos as i32 + offset).max(0) as f64
    };

    let control_1 = Point::new(
        get_ctrl_point(init_pos.x, fin_pos.x),
        get_ctrl_point(init_pos.y, fin_pos.y),
    );
    let control_2 = Point::new(
        get_ctrl_point(init_pos.x, fin_pos.x),
        get_ctrl_point(init_pos.y, fin_pos.y),
    );

    CubicBez::new(init_pos, control_1, control_2, fin_pos)
}

/// Writes a shell script containing xdotool commands to simulate mouse movement and clicks
///
/// # Arguments
///
/// * `path` - Path where the script file will be created
/// * `curve` - A cubic Bézier curve defining the mouse movement path
/// * `speed` - Speed of mouse movement, smaller values indicate higher speeds.
///
/// # Returns
///
/// * `std::io::Result<()>` - Success or failure of the file write operation
pub fn write_xdotool_script(
    path: &std::path::Path,
    curve: CubicBez,
    speed: u32,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(b"#!/bin/bash\n")?;

    // Calculate the number of points inversely proportional to speed
    // Higher speed = fewer points = faster movement
    let num_points = 25 + (1000 / speed.max(1));

    let points: Vec<_> = (0..=num_points)
        .map(|t| t as f64 / f64::from(num_points))
        .map(|t| curve.eval(t))
        .map(|point| format!("xdotool mousemove {} {}\n", point.x, point.y))
        .collect();

    for point in points {
        file.write_all(point.as_bytes())?;
    }

    // The call to sleep guarantees the mouse makes it to its final position before the click
    file.write_all(b"sleep 0.1\n")?; // Reduced sleep time for faster completion
    file.write_all(b"xdotool click 1\n")?;

    Ok(())
}

/// Executes a shell script containing xdotool commands
///
/// # Arguments
///
/// * `path` - Path to the shell script to execute
///
/// # Returns
///
/// * `Result<std::process::ExitStatus, std::io::Error>` - Exit status of the script execution or an error
pub fn run_xdotool_script(
    path: &std::path::Path,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg(path);
    cmd.status()
}

/// Compares two RGBA color values within a specified tolerance
///
/// # Arguments
///
/// * `a` - First RGBA color tuple (r, g, b, a)
/// * `b` - Second RGBA color tuple (r, g, b, a)
/// * `tolerance` - Maximum allowed difference for each color component
///
/// # Returns
///
/// `true` if all color components are within the specified tolerance, `false` otherwise
fn color_matches(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8), tolerance: u8) -> bool {
    (a.0 as i16 - b.0 as i16).abs() <= tolerance as i16
        && (a.1 as i16 - b.1 as i16).abs() <= tolerance as i16
        && (a.2 as i16 - b.2 as i16).abs() <= tolerance as i16
}

/// Captures the screen and finds all pixels matching a target color within a tolerance
///
/// # Arguments
///
/// * `target_color` - Target RGBA color tuple to search for
///
/// # Returns
///
/// * `Result<Vec<Point>, Box<dyn std::error::Error>>` - Vector of points where matching pixels were found,
///   or an error if screen capture fails
pub fn get_pixels_with_target_color(
    target_color: &(u8, u8, u8, u8),
) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    // Get the primary display
    let display = Display::primary()?;
    let width = display.width();
    let mut capturer = Capturer::new(display)?;
    let mut matches = Vec::new();
    const TOLERANCE: u8 = 3;

    loop {
        // Try to capture a frame
        if let Ok(frame) = capturer.frame() {
            // Iterate over the pixels
            for (i, pixel) in frame.chunks(4).enumerate() {
                // Pixels are in BGRA format
                let b = pixel[0];
                let g = pixel[1];
                let r = pixel[2];
                let a = pixel[3];

                if color_matches((b, g, r, a), *target_color, TOLERANCE) {
                    // Calculate pixel coordinates
                    let x = i % width;
                    let y = i / width;
                    matches.push(Point::new(x as f64, y as f64));
                }
            }

            break; // Exit after one frame
        }
    }

    Ok(matches)
}

/// Reads and parses a bot script file containing mouse events
///
/// # Arguments
///
/// * `path` - Path to the JSON script file containing mouse events
///
/// # Returns
///
/// * `Result<Vec<MouseEvent>, Box<dyn std::error::Error>>` - Vector of parsed mouse events,
///   or an error if the file cannot be read or parsed
pub fn read_bot_script(
    path: &std::path::Path,
) -> Result<Vec<MouseEvent>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let script: BotScript = serde_json::from_reader(reader)?;

    Ok(script.events.clone())
}

/// Gets the current mouse cursor position on the screen
///
/// # Returns
///
/// A `Point` struct containing the x and y coordinates of the mouse cursor
fn get_mouse_position() -> Point {
    let device_state = DeviceState::new();
    let mouse_state = device_state.get_mouse();

    Point::new(mouse_state.coords.0 as f64, mouse_state.coords.1 as f64)
}

fn calculate_centroid(boundary_points: &[Point]) -> Point {
    // Clone and work on the points
    let mut points = boundary_points.to_vec();

    // Step 1: Find the geometric center
    let center_x = points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64;
    let center_y = points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64;
    let center = Point::new(center_x, center_y);

    // Step 2: Sort points counterclockwise around the center
    points.sort_by(|p1, p2| {
        let angle1 = (p1.y - center.y).atan2(p1.x - center.x);
        let angle2 = (p2.y - center.y).atan2(p2.x - center.x);
        angle1.partial_cmp(&angle2).unwrap()
    });

    // Step 3: Ensure the shape is closed
    if points.first() != points.last() {
        points.push(points[0]);
    }

    // Step 4: Calculate the centroid using the Shoelace formula
    let mut area = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;

    for i in 0..points.len() - 1 {
        let p1 = points[i];
        let p2 = points[i + 1];
        let cross = p1.x * p2.y - p2.x * p1.y;
        area += cross;
        cx += (p1.x + p2.x) * cross;
        cy += (p1.y + p2.y) * cross;
    }

    area *= 0.5;
    cx /= 6.0 * area;
    cy /= 6.0 * area;

    Point::new(cx, cy)
}

/// Executes a single mouse event according to the specified configuration
///
/// # Arguments
///
/// * `event` - The mouse event to execute, containing color and timing information
/// * `config` - Configuration settings for the bot behavior
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - Success or an error if the event execution fails
fn execute_event(event: &MouseEvent, config: &BotConfig) -> Result<(), Box<dyn std::error::Error>> {
    debug!("executing event: {:?}", event.id);

    let mut rng = rand::thread_rng();

    if event.id == "exit bank" {
        press_escape()?;

        let delay = rng.gen_range(event.delay_rng[0]..=event.delay_rng[1]);
        debug!("sleeping for {} milliseconds", delay);
        std::thread::sleep(std::time::Duration::from_millis(u64::from(delay)));

        return Ok(());
    }

    if event.id == "press 7" {
        press_seven()?;

        let delay = rng.gen_range(event.delay_rng[0]..=event.delay_rng[1]);
        debug!("sleeping for {} milliseconds", delay);
        std::thread::sleep(std::time::Duration::from_millis(u64::from(delay)));

        return Ok(());
    }

    // Indicates we are executing a position click event
    if event.color == [0, 0, 0] {
        let mut click_pos = Point::new(event.position[0] as f64, event.position[1] as f64);
        click_pos.x += rng.gen_range(-5.0..=5.0);
        click_pos.y += rng.gen_range(-5.0..=5.0);

        let curve = mouse_bez(get_mouse_position(), click_pos, config.mouse_deviation);
        let script_path = std::path::Path::new("/tmp/colorbot.sh");
        write_xdotool_script(script_path, curve, config.mouse_speed)?;
        run_xdotool_script(script_path)?;

        let delay = rng.gen_range(event.delay_rng[0]..=event.delay_rng[1]);
        debug!("sleeping for {} milliseconds", delay);
        std::thread::sleep(std::time::Duration::from_millis(u64::from(delay)));

        return Ok(());
    }

    let matches =
        get_pixels_with_target_color(&(event.color[2], event.color[1], event.color[0], 0))?;
    if matches.is_empty() {
        warn!("no matches found for color {:?}", event.color);
    } else {
        let mut rng = rand::thread_rng();
        let delay = rng.gen_range(event.delay_rng[0]..=event.delay_rng[1]);
        let init_pos = get_mouse_position();
        let mut fin_pos = calculate_centroid(&matches);
        fin_pos.x += rng.gen_range(-5.0..=5.0);
        fin_pos.y += rng.gen_range(-5.0..=5.0);

        let curve = mouse_bez(init_pos, fin_pos, config.mouse_deviation);
        let script_path = std::path::Path::new("/tmp/colorbot.sh");

        debug!(
            "clicking on color {:?} at position {:?}",
            event.color, fin_pos
        );
        write_xdotool_script(script_path, curve, config.mouse_speed)?;
        run_xdotool_script(script_path)?;

        debug!("sleeping for {} milliseconds", delay);
        std::thread::sleep(std::time::Duration::from_millis(u64::from(delay)));
    }

    Ok(())
}

fn press_escape() -> Result<(), Box<dyn std::error::Error>> {
    let script_path = std::path::Path::new("/tmp/press_escape.sh");
    let mut file = std::fs::File::create(script_path)?;
    file.write_all(b"#!/bin/bash\n")?;
    file.write_all(b"xdotool key Escape\n")?;

    run_xdotool_script(script_path)?;

    Ok(())
}

fn press_seven() -> Result<(), Box<dyn std::error::Error>> {
    let script_path = std::path::Path::new("/tmp/press_seven.sh");
    let mut file = std::fs::File::create(script_path)?;
    file.write_all(b"#!/bin/bash\n")?;
    file.write_all(b"xdotool key 7\n")?;

    run_xdotool_script(script_path)?;

    Ok(())
}

/// Runs the main event loop of the bot for a specified duration
///
/// # Arguments
///
/// * `config` - Configuration settings for the bot behavior, including script path and runtime
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - Success or an error if the event loop execution fails
pub fn run_event_loop(config: &BotConfig) -> Result<(), Box<dyn std::error::Error>> {
    let events = read_bot_script(&config.script)?;
    let start_time = std::time::Instant::now();
    let runtime = std::time::Duration::from_secs(u64::from(config.runtime));
    let end_time = start_time + runtime;

    while std::time::Instant::now() < end_time {
        // Execute all events in sequence
        for event in &events {
            execute_event(event, config)?;
        }
    }

    Ok(())
}
