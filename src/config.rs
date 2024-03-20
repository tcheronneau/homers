use rocket::figment::providers::Serialized;
use figment::{Figment, providers::{Format, Toml, Env}};
use std::path::PathBuf;
use log::{debug, info, Level};
use serde::Deserialize;
use rocket::serde::Serialize;

use crate::providers::tautulli::Tautulli;
use crate::providers::sonarr::Sonarr;
use crate::providers::radarr::Radarr;
use crate::providers::overseerr::Overseerr;

#[derive(Debug,Deserialize, Clone, Serialize)]
pub struct Config {
    pub tautulli: Option<Tautulli>,
    pub sonarr: Option<Sonarr>,
    pub radarr: Option<Radarr>,
    pub overseerr: Option<Overseerr>,
    pub http: rocket::Config,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            tautulli: None, 
            sonarr: None, 
            radarr: None,
            overseerr: None,
            http: rocket::Config::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum Task {
    SonarrToday(Sonarr),
    SonarrMissing(Sonarr),
    Radarr(Radarr),
    Overseerr(Overseerr),
    TautulliSessionPercentage(Tautulli),
    TautulliSession(Tautulli),
    TautulliLibrary(Tautulli),
    Default,
}


pub fn read(config_file: PathBuf, log_level: Level) -> anyhow::Result<Config> {
    info!("Reading config file {config_file:?}");

    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(config_file))
        .merge((
            "http.log_level",
            match log_level {
                Level::Trace | Level::Debug => rocket::log::LogLevel::Debug,
                Level::Info | Level::Warn => rocket::log::LogLevel::Normal,
                Level::Error => rocket::log::LogLevel::Critical,
            },
        ))
        .merge(Env::prefixed("HOMERS_").split("_"))
        .extract()?;

    debug!("Read config is {:?}", config);

    Ok(config)
}

pub fn get_tasks(config: Config) -> anyhow::Result<Vec<Task>> {
    let mut tasks = Vec::new();
    if let Some(sonarr) = config.sonarr {
        let sonarr = Sonarr::new(sonarr.address, sonarr.api_key)?;
        tasks.push(Task::SonarrToday(sonarr.clone()));
        tasks.push(Task::SonarrMissing(sonarr));
    }
    if let Some(tautulli) = config.tautulli {
        let tautulli = Tautulli::new(tautulli.address, tautulli.api_key)?;
        tasks.push(Task::TautulliSessionPercentage(tautulli.clone()));
        tasks.push(Task::TautulliSession(tautulli.clone()));
        tasks.push(Task::TautulliLibrary(tautulli));
    }
    if let Some(radarr) = config.radarr {
        let radarr = Radarr::new(radarr.address, radarr.api_key)?;
        tasks.push(Task::Radarr(radarr));
    }
    if let Some(overseerr) = config.overseerr {
        let mut reqs = 20;
        if let Some(requests) = overseerr.requests {
            reqs = requests;
        }
        let overseerr = Overseerr::new(overseerr.address, overseerr.api_key, reqs)?;
        tasks.push(Task::Overseerr(overseerr));
    }
    Ok(tasks)
}
