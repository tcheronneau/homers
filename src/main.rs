use clap::{arg, command, Parser};
use rocket::{launch, Build, Rocket};
use std::path::PathBuf;

mod config;
mod http_server;
mod prometheus;
mod providers;

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

    let log_level = args
        .verbose
        .log_level()
        .expect("Log level cannot be not available");

    simple_logger::init_with_level(log_level).expect("Logging successfully initialized");
    let config = match config::read(args.config.clone(), log_level) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to read config file : {}", err);
            std::process::exit(1);
        }
    };
    http_server::configure_rocket(config).await
}
