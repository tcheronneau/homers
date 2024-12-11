use figment::providers::Serialized;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use log::{debug, info, Level};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::http_server::HttpConfig;
use crate::providers::jellyfin::Jellyfin;
use crate::providers::overseerr::Overseerr;
use crate::providers::plex::Plex;
use crate::providers::radarr::Radarr;
use crate::providers::sonarr::Sonarr;
use crate::providers::tautulli::Tautulli;

use crate::tasks::Task;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Config {
    pub tautulli: Option<Tautulli>,
    pub sonarr: Option<HashMap<String, Sonarr>>,
    pub radarr: Option<HashMap<String, Radarr>>,
    pub overseerr: Option<Overseerr>,
    pub jellyseerr: Option<Overseerr>,
    pub plex: Option<HashMap<String, Plex>>,
    pub jellyfin: Option<HashMap<String, Jellyfin>>,
    pub http: Option<HttpConfig>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            tautulli: None,
            sonarr: None,
            radarr: None,
            overseerr: None,
            jellyseerr: None,
            plex: None,
            jellyfin: None,
            http: Some(HttpConfig::default()),
        }
    }
}

pub fn read(config_file: PathBuf, log_level: Level) -> anyhow::Result<Config> {
    info!("Reading config file {config_file:?}");
    let log_level_str = match log_level {
        Level::Trace | Level::Debug => "debug",
        Level::Info => "info",
        Level::Warn => "warn",
        Level::Error => "error",
    };

    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(config_file))
        .merge(("http.log_level", log_level_str))
        .merge(Env::prefixed("HOMERS_").split("_"))
        .extract()?;

    debug!("Read config is {:?}", config);

    Ok(config)
}

fn remove_trailing_slash(s: &str) -> &str {
    if s.ends_with('/') {
        debug!("Removing trailing slash from {}", s);
        &s[..s.len() - 1]
    } else {
        s
    }
}

pub fn get_tasks(config: Config) -> anyhow::Result<Vec<Task>> {
    let mut tasks = Vec::new();
    if let Some(sonarr) = config.sonarr {
        for (name, s) in sonarr {
            let client = Sonarr::new(&name, remove_trailing_slash(&s.address), &s.api_key)?;
            tasks.push(Task::SonarrToday(client.clone()));
            tasks.push(Task::SonarrMissing(client));
        }
    }
    if let Some(tautulli) = config.tautulli {
        let tautulli = Tautulli::new(remove_trailing_slash(&tautulli.address), &tautulli.api_key)?;
        tasks.push(Task::TautulliSession(tautulli.clone()));
        tasks.push(Task::TautulliLibrary(tautulli));
    }
    if let Some(radarr) = config.radarr {
        for (name, r) in radarr {
            let client = Radarr::new(&name, remove_trailing_slash(&r.address), &r.api_key)?;
            tasks.push(Task::Radarr(client));
        }
    }
    if let Some(overseerr) = config.overseerr {
        let mut reqs = 20;
        if let Some(requests) = overseerr.requests {
            reqs = requests;
        }
        let overseerr = Overseerr::new(
            remove_trailing_slash(&overseerr.address),
            &overseerr.api_key,
            reqs,
        )?;
        tasks.push(Task::Overseerr(overseerr));
    }
    if let Some(jellyseerr) = config.jellyseerr {
        let mut reqs = 20;
        if let Some(requests) = jellyseerr.requests {
            reqs = requests;
        }
        let jellyseerr = Overseerr::new(
            remove_trailing_slash(&jellyseerr.address),
            &jellyseerr.api_key,
            reqs,
        )?;
        tasks.push(Task::Jellyseerr(jellyseerr));
    }
    if let Some(plex) = config.plex {
        for (name, p) in plex {
            let client = Plex::new(&name, remove_trailing_slash(&p.address), &p.token)?;
            tasks.push(Task::PlexSession(client.clone()));
            tasks.push(Task::PlexLibrary(client));
        }
    }
    if let Some(jellyfin) = config.jellyfin {
        for (name, j) in jellyfin {
            let client = Jellyfin::new(&name, remove_trailing_slash(&j.address), &j.api_key)?;
            tasks.push(Task::JellyfinSession(client.clone()));
            tasks.push(Task::JellyfinLibrary(client));
        }
    }
    Ok(tasks)
}
