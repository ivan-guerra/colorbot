use anyhow::{Context, Result};
use device_query::{DeviceQuery, DeviceState};
use kurbo::{CubicBez, ParamCurve, Point};
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

pub fn mouse_bez(init_pos: Point, fin_pos: Point) -> CubicBez {
    const MIN_DEVIATION: f64 = 30.0;

    let dx = fin_pos.x - init_pos.x;
    let dy = fin_pos.y - init_pos.y;
    let dist = (dx * dx + dy * dy).sqrt();

    let max_dev = dist * (MIN_DEVIATION / 50.0);
    let ctrl1_offset = rand::random_range(max_dev * 0.5..=max_dev);
    let ctrl2_offset = rand::random_range(max_dev * 0.3..=max_dev * 0.8);

    // Calculate control points with more natural curves
    let angle = dy.atan2(dx);
    let ctrl1_angle = angle + rand::random_range(-0.8..0.8);
    let ctrl2_angle = angle + rand::random_range(-0.5..0.5);

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

pub fn calculate_centroid(boundary_points: &[Point]) -> Point {
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

pub fn move_mouse(target: Point) -> Result<()> {
    const MOUSE_SETTLE_DELAY_MS: u64 = 50;
    let start_pos = get_mouse_pos();
    let target_rand = Point::new(
        target.x + f64::from(rand::random_range(-5..=5)),
        target.y + f64::from(rand::random_range(-5..=5)),
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

pub fn left_click() -> Result<()> {
    run_xdotool(&["click", "1"]).context("Failed to execute xdotool for left click")?;
    Ok(())
}

pub fn press_key(keycode: &str) -> Result<()> {
    const KEY_DELAY_MIN_MS: u64 = 100;
    const KEY_DELAY_MAX_MS: u64 = 150;

    run_xdotool(&["key", keycode])
        .context(format!("Failed to execute xdotool for key '{}'", keycode))?;

    std::thread::sleep(Duration::from_millis(rand::random_range(
        KEY_DELAY_MIN_MS..=KEY_DELAY_MAX_MS,
    )));

    Ok(())
}
