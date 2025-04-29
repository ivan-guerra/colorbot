use device_query::{DeviceQuery, DeviceState};
use kurbo::{CubicBez, ParamCurve, Point};
use log::debug;
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
#[serde(tag = "type")]
pub enum BotEvent {
    #[serde(rename = "mouse")]
    Mouse {
        id: String,
        action: String,
        color: [u8; 3],
        delay_rng: [u32; 2],
        #[serde(default = "default_count")]
        count: u32,
        #[serde(default = "default_false")]
        skip_if_vanished: bool,
    },
    #[serde(rename = "keypress")]
    KeyPress {
        id: String,
        action: String,
        delay_rng: [u32; 2],
        #[serde(default = "default_count")]
        count: u32,
    },
}

#[derive(Deserialize, Debug)]
struct BotScript {
    events: Vec<BotEvent>,
}

fn default_count() -> u32 {
    1
}

fn default_false() -> bool { false }

pub fn mouse_bez(init_pos: Point, fin_pos: Point, deviation: u32) -> CubicBez {
    let mut rng = rand::thread_rng();
    
    let dx = fin_pos.x - init_pos.x;
    let dy = fin_pos.y - init_pos.y;
    let dist = (dx * dx + dy * dy).sqrt();
    
    let max_dev = dist * (deviation as f64 / 50.0);
    let ctrl1_offset = rng.gen_range(max_dev * 0.5..=max_dev);
    let ctrl2_offset = rng.gen_range(max_dev * 0.3..=max_dev * 0.8);
    
    // Calculate control points with more natural curves
    let angle = dy.atan2(dx);
    let ctrl1_angle = angle + rng.gen_range(-0.8..0.8);
    let ctrl2_angle = angle + rng.gen_range(-0.5..0.5);
    
    let control_1 = Point::new(
        init_pos.x + ctrl1_offset * ctrl1_angle.cos(),
        init_pos.y + ctrl1_offset * ctrl1_angle.sin()
    );
    
    let control_2 = Point::new(
        fin_pos.x - ctrl2_offset * ctrl2_angle.cos(),
        fin_pos.y - ctrl2_offset * ctrl2_angle.sin()
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

    Ok(())
}

pub fn run_xdotool_script(path: &std::path::Path) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg(path);
    cmd.status()
}

fn color_matches(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8), tolerance: u8) -> bool {
    (a.0 as i16 - b.0 as i16).abs() <= tolerance as i16
        && (a.1 as i16 - b.1 as i16).abs() <= tolerance as i16
        && (a.2 as i16 - b.2 as i16).abs() <= tolerance as i16
        && (a.3 as i16 - b.3 as i16).abs() <= tolerance as i16
}

pub fn get_pixels_with_target_color(
    target_color: &(u8, u8, u8, u8),
) -> Result<Vec<(Point, u32)>, Box<dyn std::error::Error>> {
    let display = Display::primary()?;
    let width = display.width();
    let height = display.height();
    let mut capturer = Capturer::new(display)?;
    let mut matches = Vec::new();
    const TOLERANCE: u8 = 3;

    if let Ok(frame) = capturer.frame() {
        // Creates a 2D grid to count neighboring matches
        let mut density_grid = vec![vec![0; height as usize]; width as usize];
        
        // Finds all matching pixels and marks them
        for (i, pixel) in frame.chunks(4).enumerate() {
            if color_matches((pixel[0], pixel[1], pixel[2], pixel[3]), *target_color, TOLERANCE) {
                let x = i % width;
                let y = i / width;
                density_grid[x][y] = 1;
            }
        }

        // Calculate density (how many neighbors each pixel has)
        for x in 1..width-1 {
            for y in 1..height-1 {
                if density_grid[x][y] > 0 {
                    let mut density = 0;
                    // Check 8 surrounding pixels
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if density_grid[(x as i32 + dx) as usize][(y as i32 + dy) as usize] > 0 {
                                density += 1;
                            }
                        }
                    }
                    matches.push((
                        Point::new(x as f64, y as f64),
                        density
                    ));
                }
            }
        }
    }

    Ok(matches)
}

pub fn read_bot_script(path: &std::path::Path) -> Result<Vec<BotEvent>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let script: BotScript = serde_json::from_reader(reader)?;
    Ok(script.events)
}

fn get_mouse_position() -> Point {
    let device_state = DeviceState::new();
    let mouse_state = device_state.get_mouse();
    Point::new(mouse_state.coords.0 as f64, mouse_state.coords.1 as f64)
}

fn handle_click(
    action: &str,
    target: Point,
    config: &BotConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_path = match action {
        "left_click" => {
            let curve = mouse_bez(get_mouse_position(), target, config.mouse_deviation);
            let path = std::path::Path::new("/tmp/left_click.sh");
            write_xdotool_script(path, curve, config.mouse_speed)?;
            // Append left click
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)?;
            file.write_all(b"xdotool click 1\n")?;
            path
        },
        "right_click" => {
            let curve = mouse_bez(get_mouse_position(), target, config.mouse_deviation);
            let path = std::path::Path::new("/tmp/right_click.sh");
            write_xdotool_script(path, curve, config.mouse_speed)?;
            // Append right click
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)?;
            file.write_all(b"xdotool click 3\n")?;
            path
        },
        "shift_click" => {
            let curve = mouse_bez(get_mouse_position(), target, config.mouse_deviation);
            let path = std::path::Path::new("/tmp/shift_click.sh");
            // Write curve movement
            write_xdotool_script(path, curve, config.mouse_speed)?;
            // Append atomic shift+click
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)?;
            file.write_all(b"xdotool keydown Shift sleep 0.05 click 1 keyup Shift\n")?;
            path
        },
        _ => return Err("Unsupported action".into()),
    };
    
    run_xdotool_script(script_path)?;
    Ok(())
}

fn press_key(action: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script_path = std::path::Path::new("/tmp/press_key.sh");
    let mut file = std::fs::File::create(script_path)?;
    file.write_all(b"#!/bin/bash\n")?;
    file.write_all(format!("xdotool key {}\n", action).as_bytes())?;
    run_xdotool_script(script_path)?;
    Ok(())
}

fn has_other_players() -> Result<bool, Box<dyn std::error::Error>> {
    let color = (255, 255, 0, 0);
    let matches = get_pixels_with_target_color(&color)?;
    Ok(!matches.is_empty())
}

fn execute_event(event: &BotEvent, config: &BotConfig) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        BotEvent::Mouse { id, action, color, delay_rng, count, skip_if_vanished } => {
            debug!("Executing mouse event: {}", id);
            let target_color = (color[2], color[1], color[0], 0);
            let mut rng = rand::thread_rng();
            
            'attempt_loop: for _ in 0..*count {
                // Full screen check with density calculation
                let matches = match get_pixels_with_target_color(&target_color) {
                    Ok(m) if !m.is_empty() => {
                        // Sort by density (highest first) and take top 25%
                        let mut sorted = m.clone();
                        sorted.sort_by(|a, b| b.1.cmp(&a.1));
                        let top_quarter = (sorted.len() / 4).max(1);
                        sorted[..top_quarter].to_vec()
                    },
                    _ => {
                        if *skip_if_vanished {
                            debug!("Color not found on screen, skipping {}", id);
                            std::thread::sleep(std::time::Duration::from_millis(
                                rng.gen_range(200..1500) as u64
                            ));
                            continue 'attempt_loop;
                        }
                        Vec::new()
                    }
                };

                if !matches.is_empty() {
                    // Pick random target from high-density areas
                    let target = matches[rng.gen_range(0..matches.len())].0;
                    handle_click(action, target, config)?;

                    // Continuous monitoring
                    let delay_ms = rng.gen_range(delay_rng[0]..=delay_rng[1]) as u64;
                    let check_interval = 100u64;
                    let mut elapsed = 0u64;
                    
                    while elapsed < delay_ms {
                        if *skip_if_vanished {
                            match get_pixels_with_target_color(&target_color) {
                                Ok(m) if m.is_empty() => {
                                    debug!("Color vanished during wait, skipping {}", id);
                                    std::thread::sleep(std::time::Duration::from_millis(
                                        rng.gen_range(200..1500) as u64
                                    ));
                                    continue 'attempt_loop;
                                },
                                Err(e) => debug!("Scan error: {}", e),
                                _ => {}
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(check_interval));
                        elapsed += check_interval;
                    }
                } else if !*skip_if_vanished {
                    // If we're not skipping and found no matches, wait full delay
                    std::thread::sleep(std::time::Duration::from_millis(
                        rng.gen_range(delay_rng[0]..=delay_rng[1]) as u64
                    ));
                }
            }
        },
        BotEvent::KeyPress { id, action, delay_rng, count } => {
            debug!("Executing keypress: {}", id);
            let mut rng = rand::thread_rng();
            for _ in 0..*count {
                press_key(action)?;
                let delay = rng.gen_range(delay_rng[0]..=delay_rng[1]);
                std::thread::sleep(std::time::Duration::from_millis(delay.into()));
            }
        },
    }
    Ok(())
}

pub fn run_event_loop(config: &BotConfig) -> Result<(), Box<dyn std::error::Error>> {
    let events = read_bot_script(&config.script)?;
    let start_time = std::time::Instant::now();
    let runtime = std::time::Duration::from_secs(u64::from(config.runtime));
    let end_time = start_time + runtime;

    while has_other_players()? {
        press_key("ctrl+shift+Right")?;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    while std::time::Instant::now() < end_time {
        for event in &events {
            execute_event(event, config)?;
        }
    }

    Ok(())
}
