use anyhow::{bail, Context, Result};
use device_query::{DeviceQuery, DeviceState};
use kurbo::{CubicBez, ParamCurve, Point};
use log::debug;
use rand::random_range;
use scrap::{Capturer, Display};
use serde::Deserialize;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct BotConfig {
    pub script: PathBuf,
    pub runtime: u64,
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum BotEvent {
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

fn read_bot_script(path: &Path) -> Result<Vec<BotEvent>> {
    let file = File::open(path).context("Failed to open bot script")?;
    let reader = BufReader::new(file);
    let events: Vec<BotEvent> =
        serde_json::from_reader(reader).context("Failed to parse bot script")?;

    Ok(events)
}

fn run_xdotool(args: &[&str]) -> Result<()> {
    Command::new("xdotool")
        .args(args)
        .output()
        .context(format!("Failed to execute xdotool with args: {:?}", args))?;
    Ok(())
}

pub fn mouse_bez(init_pos: Point, fin_pos: Point) -> CubicBez {
    const MIN_DEVIATION: f64 = 30.0;

    let dx = fin_pos.x - init_pos.x;
    let dy = fin_pos.y - init_pos.y;
    let dist = (dx * dx + dy * dy).sqrt();

    let max_dev = dist * (MIN_DEVIATION / 50.0);
    let ctrl1_offset = random_range(max_dev * 0.5..=max_dev);
    let ctrl2_offset = random_range(max_dev * 0.3..=max_dev * 0.8);

    // Calculate control points with more natural curves
    let angle = dy.atan2(dx);
    let ctrl1_angle = angle + random_range(-0.8..0.8);
    let ctrl2_angle = angle + random_range(-0.5..0.5);

    let ctrl1 = Point::new(
        init_pos.x + ctrl1_offset * ctrl1_angle.cos(),
        init_pos.y + ctrl1_offset * ctrl1_angle.sin(),
    );
    let ctrl2 = Point::new(
        fin_pos.x - ctrl2_offset * ctrl2_angle.cos(),
        fin_pos.y - ctrl2_offset * ctrl2_angle.sin(),
    );

    CubicBez::new(init_pos, ctrl1, ctrl2, fin_pos)
}

fn color_matches(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8), tolerance: u8) -> bool {
    [a.0, a.1, a.2]
        .iter()
        .zip([b.0, b.1, b.2])
        .all(|(x, y)| (i16::from(*x) - i16::from(y)).abs() <= i16::from(tolerance))
}

pub fn get_pixels_with_target_color(target_color: &(u8, u8, u8, u8)) -> Result<Vec<Point>> {
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

fn get_mouse_pos() -> Point {
    let device_state = DeviceState::new();
    let mouse_state = device_state.get_mouse();

    Point::new(mouse_state.coords.0.into(), mouse_state.coords.1.into())
}

fn move_mouse(target: Point) -> Result<()> {
    const MOUSE_SETTLE_DELAY_MS: u64 = 50;
    let start_pos = get_mouse_pos();
    let target_rand = Point::new(
        target.x + f64::from(random_range(-5..=5)),
        target.y + f64::from(random_range(-5..=5)),
    );
    let curve = mouse_bez(start_pos, target_rand);
    let points: Vec<Point> = (0..=100)
        .map(|t| f64::from(t) / 100.0)
        .map(|t| curve.eval(t))
        .collect();

    debug!(
        "Moving mouse from ({:.1}, {:.1}) to ({:.1}, {:.1})",
        start_pos.x, start_pos.y, target_rand.x, target_rand.y
    );
    for point in points {
        let x = point.x.round() as i32;
        let y = point.y.round() as i32;

        run_xdotool(&["mousemove", &x.to_string(), &y.to_string()]).context(format!(
            "Failed to execute xdotool for mouse move to ({}, {})",
            point.x, point.y
        ))?;
    }

    std::thread::sleep(Duration::from_millis(MOUSE_SETTLE_DELAY_MS));

    Ok(())
}

fn left_click() -> Result<()> {
    run_xdotool(&["click", "1"]).context("Failed to execute xdotool for left click")?;
    Ok(())
}

fn press_key(keycode: &str) -> Result<()> {
    const KEY_DELAY_MIN_MS: u64 = 100;
    const KEY_DELAY_MAX_MS: u64 = 150;

    run_xdotool(&["key", keycode])
        .context(format!("Failed to execute xdotool for key '{}'", keycode))?;

    std::thread::sleep(Duration::from_millis(random_range(
        KEY_DELAY_MIN_MS..=KEY_DELAY_MAX_MS,
    )));

    Ok(())
}

fn drop_inventory() -> Result<()> {
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

fn canifis_recovery() -> Result<()> {
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
            exec_event(&event)?;
        }
    }
    Ok(())
}

fn logout() -> Result<()> {
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
        exec_event(&event)?;
    }
    Ok(())
}

fn exec_event(event: &BotEvent) -> Result<()> {
    let sleep_random_delay = |delay_rng: &[u32; 2]| {
        let delay = random_range(delay_rng[0]..=delay_rng[1]);
        debug!("Sleeping for {} ms", delay);
        std::thread::sleep(Duration::from_millis(delay.into()));
    };

    match event {
        BotEvent::Mouse { id, pos, delay_rng } => {
            debug!("Executing mouse event '{}' at ({}, {})", id, pos[0], pos[1]);
            let point = Point::new(pos[0].into(), pos[1].into());

            move_mouse(point)?;
            left_click()?;
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
                press_key(keycode)?;
            }
            sleep_random_delay(delay_rng);
        }
        BotEvent::Color { id, rgb, delay_rng } => {
            debug!(
                "Executing color event '{}': target RGB({},{},{})",
                id, rgb[0], rgb[1], rgb[2]
            );
            let target_bgra = (rgb[2], rgb[1], rgb[0], 0);
            let matches = get_pixels_with_target_color(&target_bgra)?;

            if matches.is_empty() {
                logout().context("Failed to execute logout sequence after color event failure")?;
                bail!("No matching pixels found for event '{}', logged out", id);
            } else {
                debug!("Found {} matching pixels for event '{}'", matches.len(), id);
                let centroid = calculate_centroid(&matches);

                move_mouse(centroid)?;
                left_click()?;
                sleep_random_delay(delay_rng);
            }
        }
        BotEvent::SpecialAction { id } => {
            debug!("Executing special action '{}'", id);
            if id == "drop_inventory" {
                drop_inventory().context("Failed to execute inventory drop action")?;
            } else if id == "canifis_recovery" {
                canifis_recovery().context("Failed to execute Canifis recovery action")?;
            } else {
                bail!("Unknown special action '{}'", id);
            }
        }
    }
    Ok(())
}

pub fn run_event_loop(config: &BotConfig) -> Result<()> {
    let events = read_bot_script(&config.script)?;
    debug!("Loaded {} events from script", events.len());

    let runtime = Duration::from_secs(config.runtime);
    let start_time = Instant::now();
    let end_time = start_time + runtime;
    debug!("Starting event loop for {} seconds", config.runtime);

    let mut iteration = 0;
    while Instant::now() < end_time {
        debug!("Starting iteration {}", iteration);

        for event in &events {
            exec_event(event)?;
        }
        iteration += 1;
    }

    debug!("Event loop completed after {} iterations", iteration);
    Ok(())
}
