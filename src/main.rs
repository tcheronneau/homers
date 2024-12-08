use clap::{arg, command, Parser};
use rocket::{launch, Build, Rocket};
use std::path::PathBuf;

mod config;
mod http_server;
mod prometheus;
mod providers;
mod tasks;

#[cfg(debug_assertions)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DebugLevel;

#[cfg(debug_assertions)]
impl clap_verbosity_flag::LogLevel for DebugLevel {
    fn default() -> Option<log::Level> {
        Some(log::Level::Debug)
    }
}

#[cfg(debug_assertions)]
type DefaultLogLevel = DebugLevel;

#[cfg(not(debug_assertions))]
type DefaultLogLevel = clap_verbosity_flag::WarnLevel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity<DefaultLogLevel>,
    #[arg(short, long)]
    config: PathBuf,
}

#[launch]
pub async fn start_server() -> Rocket<Build> {
    let args = Args::parse();

    let log_level = match args.verbose.log_level() {
        Some(level) => level,
        None => log::Level::Info,
    };

    match simple_logger::init_with_level(log_level) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Failed to initialize logger: {}", err);
            std::process::exit(1);
        }
    };
    let config = match config::read(args.config.clone(), log_level) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to read config file : {}", err);
            std::process::exit(1);
        }
    };
    http_server::configure_rocket(config).await
}
