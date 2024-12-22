use clap::{arg, command, Parser};
use log::Level;
use std::path::PathBuf;
use tokio::signal::unix::{signal, SignalKind};

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
type DefaultLogLevel = clap_verbosity_flag::InfoLevel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity<DefaultLogLevel>,
    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let log_level = args.verbose.log_level().unwrap_or(log::Level::Info);
    setup_logging(log_level);

    let config = match config::read(args.config.clone(), log_level) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to read config file : {}", err);
            std::process::exit(1);
        }
    };
    tokio::select! {
        _ = handle_shutdown_signal() => {}
        _ = run_server(config) => {}
    }
}
async fn run_server(config: config::Config) {
    if let Err(err) = http_server::configure_axum(config).await {
        eprintln!("Failed to start server: {}", err);
        std::process::exit(1);
    }
}
async fn handle_shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to create SIGTERM listener");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to create SIGINT listener");

    tokio::select! {
        _ = sigterm.recv() => {
            log::warn!("Exiting...");
        }
        _ = sigint.recv() => {
            log::warn!("Interrupting...");
        }
    }
}
fn setup_logging(log_level: Level) {
    //tracing_log::LogTracer::init().expect("Failed to set up log compatibility");

    // Configure tracing subscriber
    let env_filter = tracing_subscriber::EnvFilter::default().add_directive(
        log_level
            .to_string()
            .parse()
            .unwrap_or_else(|_| "info".parse().unwrap()),
    );

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
}
