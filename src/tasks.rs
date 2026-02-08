use prometheus_client::registry::Registry;
use serde::Deserialize;

use crate::prometheus::FormatAsPrometheus;
use crate::providers::jellyfin::Jellyfin;
use crate::providers::lidarr::{Lidarr, LidarrArtist};
use crate::providers::overseerr::{Overseerr, OverseerrRequest};
use crate::providers::plex::Plex;
use crate::providers::radarr::{Radarr, RadarrMovie};
use crate::providers::readarr::{Readarr, ReadarrAuthor};
use crate::providers::sonarr::{Sonarr, SonarrEpisode};
use crate::providers::structs::tautulli::Library;
use crate::providers::structs::{LibraryCount, Session, User};
use crate::providers::tautulli::{HistoryEntry, SessionSummary};
use crate::providers::tautulli::Tautulli;

#[derive(Debug, Deserialize, Clone)]
pub enum Task {
    SonarrToday(Sonarr),
    SonarrMissing(Sonarr),
    Radarr(Radarr),
    Readarr(Readarr),
    Lidarr(Lidarr),
    Overseerr(Overseerr),
    Jellyseerr(Overseerr),
    TautulliSession(Tautulli),
    TautulliLibrary(Tautulli),
    TautulliHistory(Tautulli),
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
    TautulliHistory(TautulliHistoryResult),
    Radarr(RadarrMovieResult),
    Readarr(ReadarrAuthorResult),
    Lidarr(LidarrArtistResult),
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
            TaskResult::TautulliHistory(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Radarr(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Readarr(result) => {
                result.format_as_prometheus(registry);
            }
            TaskResult::Lidarr(result) => {
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
pub struct TautulliHistoryResult {
    pub total_plays: i64,
    pub entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone)]
pub struct RadarrMovieResult {
    pub name: String,
    pub movies: Vec<RadarrMovie>,
}

#[derive(Debug, Clone)]
pub struct ReadarrAuthorResult {
    pub name: String,
    pub authors: Vec<ReadarrAuthor>,
}

#[derive(Debug, Clone)]
pub struct LidarrArtistResult {
    pub name: String,
    pub artists: Vec<LidarrArtist>,
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
