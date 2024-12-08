use serde::Deserialize;

use crate::providers::jellyfin::Jellyfin;
use crate::providers::overseerr::{Overseerr, OverseerrRequest};
use crate::providers::plex::Plex;
use crate::providers::radarr::{Radarr, RadarrMovie};
use crate::providers::sonarr::{Sonarr, SonarrEpisode};
use crate::providers::structs::tautulli::Library;
use crate::providers::structs::{
    BandwidthLocation, LibraryCount, MediaType as LibraryMediaType, Session, User,
};
use crate::providers::tautulli::SessionSummary;
use crate::providers::tautulli::Tautulli;

use std::collections::HashMap;

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
    Overseerr(Vec<OverseerrRequest>),
    Jellyseerr(Vec<OverseerrRequest>),
    PlexSession(SessionResult),
    PlexLibrary(HashMap<String, Vec<LibraryCount>>),
    JellyfinSession(SessionResult),
    JellyfinLibrary(HashMap<String, Vec<LibraryCount>>),
    Default,
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
pub struct SessionResult {
    pub name: String,
    pub kind: String,
    pub users: Vec<User>,
    pub sessions: Vec<Session>,
}
