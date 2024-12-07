use kurbo::{CubicBez, ParamCurve, Point};
use rand::Rng;
use scrap::{Capturer, Display};
use serde::Deserialize;
use std::io::Write;

pub struct BotConfig {
    pub script: std::path::PathBuf,
    pub runtime: u32,
    pub mouse_deviation: u32,
    pub mouse_speed: u32,
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

#[derive(Deserialize, Debug, Clone)]
pub struct MouseEvent {
    pub id: String,
    pub color: [u8; 3],
    pub delay_rng: [u32; 2],
}

#[derive(Deserialize, Debug)]
struct BotScript {
    events: Vec<MouseEvent>,
}

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

pub fn write_xdotool_script(
    path: &std::path::Path,
    curve: CubicBez,
    speed: u32,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(b"#!/bin/bash\n")?;

    let points: Vec<_> = (0..=speed * 100)
        .map(|t| t as f64 / f64::from(speed * 100))
        .map(|t| curve.eval(t))
        .map(|point| format!("xdotool mousemove {} {}\n", point.x, point.y))
        .collect();

    for point in points {
        file.write_all(point.as_bytes())?;
    }

    file.write_all(b"xdotool click 1\n")?;

    Ok(())
}

pub fn run_xdotool_script(
    path: &std::path::Path,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg(path);
    cmd.status()
}

pub fn get_pixels_with_target_color(
    target_color: &(u8, u8, u8, u8),
) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    // Get the primary display
    let display = Display::primary()?;
    let width = display.width();
    let mut capturer = Capturer::new(display)?;
    let mut matches = Vec::new();

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

                if (b, g, r, a) == *target_color {
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

pub fn read_bot_script(
    path: &std::path::Path,
) -> Result<Vec<MouseEvent>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let script: BotScript = serde_json::from_reader(reader)?;

    Ok(script.events.clone())
}
