use figment::{Figment, providers::{Format, Toml, Env}};
use std::path::PathBuf;
use log::{debug, info, Level};
use serde::Deserialize;

use crate::tautulli::Tautulli;
use crate::sonarr::Sonarr;

#[derive(Debug,Deserialize)]
pub struct Config <'a> {
    pub tautulli: Tautulli,
    #[serde(borrow)]
    pub sonarr: Sonarr<'a>,
    //pub http: rocket::Config,
}

pub fn read(config_file: PathBuf, log_level: Level) -> anyhow::Result<Config<'static>> {
    info!("Reading config file {config_file:?}");

    let config: Config = Figment::new()
        //.merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(config_file))
        //.merge((
        //    "http.log_level",
        //    match log_level {
        //        Level::Trace | Level::Debug => rocket::log::LogLevel::Debug,
        //        Level::Info | Level::Warn => rocket::log::LogLevel::Normal,
        //        Level::Error => rocket::log::LogLevel::Critical,
        //    },
        //))
        .merge(Env::prefixed("HOMERS_").split("_"))
        .extract()?;

    debug!("Read config is {:?}", config);

    Ok(config)
}
