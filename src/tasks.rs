use prometheus_client::registry::Registry;
use serde::Deserialize;

use crate::prometheus::FormatAsPrometheus;
use crate::providers::jellyfin::Jellyfin;
use crate::providers::overseerr::{Overseerr, OverseerrRequest};
use crate::providers::plex::Plex;
use crate::providers::radarr::{Radarr, RadarrMovie};
use crate::providers::sonarr::{Sonarr, SonarrEpisode};
use crate::providers::structs::tautulli::Library;
use crate::providers::structs::{LibraryCount, Session, User};
use crate::providers::tautulli::SessionSummary;
use crate::providers::tautulli::Tautulli;

#[derive(Debug, Deserialize, Clone)]
pub enum Task {
    SonarrToday(Sonarr),
    SonarrMissing(Sonarr),
    Radarr(Radarr),
    Overseerr(Overseerr),
    Jellyseerr(Overseerr),
    TautulliSession(Tautulli),
    TautulliLibrary(Tautulli),
    PlexSession(Plex),
    PlexLibrary(Plex),
    JellyfinSession(Jellyfin),
    JellyfinLibrary(Jellyfin),
    Default,
}
pub enum TaskResult {
    SonarrToday(SonarrEpisodeResult),
    SonarrMissing(SonarrMissingResult),
    TautulliSession(TautulliSessionResult),
    TautulliLibrary(TautulliLibraryResult),
    Radarr(RadarrMovieResult),
    Overseerr(OverseerrRequestResult),
    Jellyseerr(OverseerrRequestResult),
    PlexSession(SessionResult),
    PlexLibrary(LibraryResult),
    JellyfinSession(SessionResult),
    JellyfinLibrary(LibraryResult),
    Default,
}
impl FormatAsPrometheus for TaskResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        match self {
            TaskResult::SonarrToday(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::SonarrMissing(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::TautulliSession(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::TautulliLibrary(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Radarr(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Overseerr(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Jellyseerr(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::PlexSession(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::PlexLibrary(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::JellyfinSession(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::JellyfinLibrary(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Default => {}
        }
    }
}
#[derive(Clone, Debug)]
pub struct SonarrEpisodeResult {
    pub name: String,
    pub episodes: Vec<SonarrEpisode>,
}
#[derive(Clone, Debug)]
pub struct SonarrMissingResult {
    pub name: String,
    pub episodes: Vec<SonarrEpisode>,
}

#[derive(Debug, Clone)]
pub struct TautulliSessionResult {
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone)]
pub struct TautulliLibraryResult {
    pub libraries: Vec<Library>,
}

#[derive(Debug, Clone)]
pub struct RadarrMovieResult {
    pub name: String,
    pub movies: Vec<RadarrMovie>,
}

#[derive(Debug, Clone)]
pub struct OverseerrRequestResult {
    pub kind: String,
    pub requests: Vec<OverseerrRequest>,
}

#[derive(Debug, Clone)]
pub struct SessionResult {
    pub name: String,
    pub kind: String,
    pub users: Vec<User>,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Clone)]
pub struct LibraryResult {
    pub name: String,
    pub kind: String,
    pub libraries: Vec<LibraryCount>,
}
