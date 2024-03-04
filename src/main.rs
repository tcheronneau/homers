use rocket::{launch, Build, Rocket};
use clap::{arg, command, Parser};
use std::path::PathBuf;

mod http_server;
mod config;
mod providers;
mod prometheus;

use crate::providers::sonarr::Sonarr;

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

//#[tokio::main]
//async fn main() {
    //let tautulli = Tautulli::new(config.tautulli.address, config.tautulli.api_key);
    //let sonarr = Sonarr::new(config.sonarr.address, config.sonarr.api_key);
    //let session_summaries = tautulli.get_session_summary().await;

    //
    //for item in session_summaries.expect("Failed to get session summaries") {
    //    println!("{}", item);
    //}
    //let shows = sonarr.get_today_shows().await;
    ////let status = sonarr.debug("system/status").await;
    //for item in shows {
    //    println!("{}", item);
    //}
//}
#[launch]
pub async fn start_server() -> Rocket<Build> {
    let args = Args::parse();

    let log_level = args
        .verbose
        .log_level()
        .expect("Log level cannot be not available");

    simple_logger::init_with_level(log_level).expect("Logging successfully initialized");
    let config = config::read(args.config.clone(), log_level).expect("Config successfully read");
    http_server::configure_rocket(config).await
    //let sonarr_config = config.sonarr.clone().expect("Sonarr config not found");
    //let sonarr = Sonarr::new(sonarr_config.address, sonarr_config.api_key);
    //let shows = sonarr.get_today_shows();
    //for item in shows {
    //    println!("{}", item);
    //}
    //let tautulli = Tautulli::new(config.tautulli.address, config.tautulli.api_key);
    //let sonarr = Sonarr::new(config.sonarr.address, config.sonarr.api_key);
    //let session_summaries = tautulli.get_session_summary().await;

    //
    //for item in session_summaries.expect("Failed to get session summaries") {
    //    println!("{}", item);
    //}
    //let shows = sonarr.get_today_shows().await;
    ////let status = sonarr.debug("system/status").await;
    //for item in shows {
    //    println!("{}", item);
    //}
}
