fn main() {
    println!("Hello, world!");
    let bez = colorbot::mouse_bez(
        kurbo::Point::new(0.0, 0.0),
        kurbo::Point::new(100.0, 1000.0),
        10,
    );
    colorbot::write_xdotool_script(std::path::Path::new("/tmp/rsbot.sh"), bez, 3).unwrap();
    colorbot::run_xdotool_script(std::path::Path::new("/tmp/rsbot.sh")).unwrap();
}
