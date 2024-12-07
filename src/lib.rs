use kurbo::{CubicBez, ParamCurve, Point};
use rand::Rng;
use std::io::Write;

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
