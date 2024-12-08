use clap::Parser;
use simplelog::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "path to bot script")]
    script: std::path::PathBuf,

    #[arg(
        short = 'r',
        long,
        default_value_t = 3600,
        help = "script runtime in seconds"
    )]
    runtime: u32,

    #[arg(
        short = 'd',
        long,
        default_value_t = 30,
        value_parser = clap::value_parser!(u32).range(0..=100),
        help = "determines the deviation of the mouse during pathing"
    )]
    mouse_deviation: u32,

    #[arg(
        short = 's',
        long,
        default_value_t = 3,
        value_parser = clap::value_parser!(u32).range(1..=10),
        help = "defines the speed of the mouse, lower means faster"
    )]
    mouse_speed: u32,

    #[arg(short = 'g', long, default_value_t = false, help = "enable logging")]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let config = colorbot::BotConfig::new(
        args.script,
        args.runtime,
        args.mouse_deviation,
        args.mouse_speed,
        args.debug,
    );

    if config.debug {
        if let Err(e) = TermLogger::init(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = colorbot::run_event_loop(&config) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
