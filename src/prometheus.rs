use log::debug;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use crate::providers::overseerr::OverseerrRequest;
use crate::providers::radarr::RadarrMovie;
use crate::providers::sonarr::SonarrEpisode;
use crate::providers::structs::tautulli::Library;
use crate::providers::structs::{
    BandwidthLocation, LibraryCount, MediaType as LibraryMediaType, Session, User,
};
use crate::providers::tautulli::SessionSummary;

#[derive(PartialEq, Debug, Eq, Copy, Clone)]
pub enum Format {
    Prometheus,
    OpenMetrics,
}

pub enum TaskResult {
    SonarrToday(HashMap<String, Vec<SonarrEpisode>>),
    SonarrMissing(HashMap<String, Vec<SonarrEpisode>>),
    TautulliSession(Vec<SessionSummary>),
    TautulliLibrary(Vec<Library>),
    Radarr(HashMap<String, Vec<RadarrMovie>>),
    Overseerr(Vec<OverseerrRequest>),
    Jellyseerr(Vec<OverseerrRequest>),
    PlexSession(HashMap<String, Vec<Session>>, Vec<User>),
    PlexLibrary(HashMap<String, Vec<LibraryCount>>),
    JellyfinSession(HashMap<String, Vec<Session>>, Vec<User>),
    JellyfinLibrary(HashMap<String, Vec<LibraryCount>>),
    Default,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionBandwidth {
    pub name: String,
    pub location: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionLabels {
    pub name: String,
    pub title: String,
    pub user: String,
    pub decision: String,
    pub state: String,
    pub platform: String,
    pub local: i8,
    pub relayed: i8,
    pub secure: i8,
    pub address: String,
    pub media_type: String,
    pub public_address: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub quality: String,
    pub city: String,
    pub longitude: String,
    pub latitude: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexShowLabels {
    pub name: String,
    pub library_name: String,
    pub library_type: String,
    pub season_count: Option<i64>,
    pub episode_count: Option<i64>,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexLibraryLabels {
    pub name: String,
    pub library_name: String,
    pub library_type: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SonarrLabels {
    pub name: String,
    pub sxe: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub serie: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexCount {
    pub name: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliSessionPercentageLabels {
    pub user: String,
    pub title: String,
    pub state: String,
    pub media_type: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub video_stream: String,
    pub quality: String,
    pub quality_profile: String,
    pub city: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct EmptyLabel {}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliSessionLabels {
    pub user: String,
    pub title: String,
    pub state: String,
    pub media_type: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub video_stream: String,
    pub quality: String,
    pub quality_profile: String,
    pub city: String,
    pub longitude: String,
    pub latitude: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliLibraryLabels {
    pub section_name: String,
    pub section_type: String,
    pub count: String,
    pub parent_count: Option<String>,
    pub child_count: Option<String>,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct RadarrLabels {
    pub name: String,
    pub title: String,
    pub is_available: i8,
    pub monitored: i8,
    pub missing_available: i8,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct OverseerrLabels {
    pub media_type: String,
    pub requested_by: String,
    pub request_status: String,
    pub media_status: String,
    pub media_title: String,
    pub requested_at: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct OverseerrRequestsLabels {
    kind: String,
}

pub fn format_metrics(task_result: Vec<TaskResult>) -> anyhow::Result<String> {
    let mut buffer = String::new();
    let mut registry = Registry::with_prefix("homers");
    for task_result in task_result {
        match task_result {
            TaskResult::SonarrToday(sonarr_hash) => {
                format_sonarr_today_metrics(sonarr_hash, &mut registry)
            }
            TaskResult::SonarrMissing(sonarr_hash) => {
                format_sonarr_missing_metrics(sonarr_hash, &mut registry)
            }
            TaskResult::TautulliSession(sessions) => {
                format_tautulli_session_metrics(sessions, &mut registry)
            }
            TaskResult::TautulliLibrary(libraries) => {
                format_tautulli_library_metrics(libraries, &mut registry)
            }
            TaskResult::Radarr(movies) => format_radarr_metrics(movies, &mut registry),
            TaskResult::Overseerr(overseerr) => {
                format_overseerr_metrics("overseerr", overseerr, &mut registry)
            }
            TaskResult::Jellyseerr(overseerr) => {
                format_overseerr_metrics("jellyseerr", overseerr, &mut registry)
            }
            TaskResult::PlexSession(sessions, users) => {
                format_session_metrics("plex", sessions, users, &mut registry)
            }
            TaskResult::PlexLibrary(libraries) => {
                format_library_metrics("plex", libraries, &mut registry)
            }
            TaskResult::JellyfinSession(sessions, users) => {
                format_session_metrics("jellyfin", sessions, users, &mut registry)
            }
            TaskResult::JellyfinLibrary(libraries) => {
                format_library_metrics("jellyfin", libraries, &mut registry)
            }
            TaskResult::Default => return Err(anyhow::anyhow!("No task result")),
        }
    }
    encode(&mut buffer, &registry)?;
    Ok(buffer)
}

fn format_sonarr_today_metrics(
    sonarr_hash: HashMap<String, Vec<SonarrEpisode>>,
    registry: &mut Registry,
) {
    debug!("Formatting {sonarr_hash:?} as Prometheus");
    let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "sonarr_today_episode",
        format!("Sonarr today episode status"),
        sonarr_episode.clone(),
    );
    sonarr_hash.into_iter().for_each(|(name, episode)| {
        episode.into_iter().for_each(|ep: SonarrEpisode| {
            let labels = SonarrLabels {
                name: name.clone(),
                sxe: ep.sxe.clone(),
                season_number: ep.season_number,
                episode_number: ep.episode_number,
                title: ep.title.clone(),
                serie: ep.serie.clone(),
            };
            sonarr_episode
                .get_or_create(&labels)
                .set(if ep.has_file { 1.0 } else { 0.0 });
        });
    });
}
fn format_sonarr_missing_metrics(
    sonarr_hash: HashMap<String, Vec<SonarrEpisode>>,
    registry: &mut Registry,
) {
    debug!("Formatting {sonarr_hash:?} as Prometheus");
    let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "sonarr_missing_episode",
        format!("Sonarr missing episode status"),
        sonarr_episode.clone(),
    );
    sonarr_hash.into_iter().for_each(|(name, episode)| {
        episode.into_iter().for_each(|ep: SonarrEpisode| {
            let labels = SonarrLabels {
                name: name.clone(),
                sxe: ep.sxe.clone(),
                season_number: ep.season_number,
                episode_number: ep.episode_number,
                title: ep.title.clone(),
                serie: ep.serie.clone(),
            };
            sonarr_episode
                .get_or_create(&labels)
                .set(if ep.has_file { 1.0 } else { 0.0 });
        });
    });
}
fn format_tautulli_session_metrics(sessions: Vec<SessionSummary>, registry: &mut Registry) {
    debug!("Formatting {sessions:?} as Prometheus");
    let tautulli_session = Family::<TautulliSessionLabels, Gauge<f64, AtomicU64>>::default();
    let tautulli_session_percentage =
        Family::<TautulliSessionPercentageLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "tautulli_session",
        format!("Tautulli session status"),
        tautulli_session.clone(),
    );
    registry.register(
        "tautulli_session_percentage",
        format!("Tautulli session progress"),
        tautulli_session_percentage.clone(),
    );
    sessions.into_iter().for_each(|session| {
        let labels = TautulliSessionPercentageLabels {
            user: session.user.clone(),
            title: session.title.clone(),
            state: session.state.clone(),
            media_type: session.media_type.clone(),
            season_number: session.season_number.clone(),
            episode_number: session.episode_number.clone(),
            quality: session.quality.clone(),
            quality_profile: session.quality_profile.clone(),
            video_stream: session.video_stream.clone(),
            city: session.location.city.clone(),
        };
        tautulli_session_percentage
            .get_or_create(&labels)
            .set(session.progress.parse::<f64>().unwrap_or(0.0));
        let labels = TautulliSessionLabels {
            user: session.user.clone(),
            title: session.title.clone(),
            state: session.state.clone(),
            media_type: session.media_type.clone(),
            season_number: session.season_number.clone(),
            episode_number: session.episode_number.clone(),
            quality: session.quality.clone(),
            quality_profile: session.quality_profile.clone(),
            video_stream: session.video_stream.clone(),
            city: session.location.city.clone(),
            longitude: session.location.longitude.clone(),
            latitude: session.location.latitude.clone(),
        };
        tautulli_session.get_or_create(&labels).set(1.0);
    });
}
fn format_tautulli_library_metrics(libraries: Vec<Library>, registry: &mut Registry) {
    debug!("Formatting {libraries:?} as Prometheus");
    let tautulli_library = Family::<TautulliLibraryLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "tautulli_library",
        format!("Tautulli library status"),
        tautulli_library.clone(),
    );
    libraries.into_iter().for_each(|library| {
        let labels = TautulliLibraryLabels {
            section_name: library.section_name.clone(),
            section_type: library.section_type.clone(),
            count: library.count.clone(),
            parent_count: library.parent_count.clone(),
            child_count: library.child_count.clone(),
        };
        tautulli_library
            .get_or_create(&labels)
            .set(library.is_active as f64);
    });
}

fn format_radarr_metrics(radarr_hash: HashMap<String, Vec<RadarrMovie>>, registry: &mut Registry) {
    debug!("Formatting {radarr_hash:?} as Prometheus");
    let radarr_movie = Family::<RadarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "radarr_movie",
        format!("Radarr movie status"),
        radarr_movie.clone(),
    );
    radarr_hash.into_iter().for_each(|(name, movies)| {
        movies.into_iter().for_each(|movie| {
            let labels = RadarrLabels {
                name: name.clone(),
                title: movie.title.clone(),
                is_available: movie.is_available as i8,
                monitored: movie.monitored as i8,
                missing_available: movie.missing_available as i8,
            };
            radarr_movie
                .get_or_create(&labels)
                .set(if movie.has_file { 1.0 } else { 0.0 });
        });
    });
}

fn format_overseerr_metrics(kind: &str, requests: Vec<OverseerrRequest>, registry: &mut Registry) {
    debug!("Formatting {requests:?} as Prometheus");
    let overseerr_request = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        format!("{}_requests", kind),
        format!("{} requests status", kind),
        overseerr_request.clone(),
    );

    requests.into_iter().for_each(|request| {
        let labels = OverseerrLabels {
            media_type: request.media_type.clone(),
            requested_by: request.requested_by.to_string(),
            request_status: request.status.to_string(),
            media_status: request.media_status.to_string(),
            media_title: request.media_title,
            requested_at: request.requested_at,
        };
        overseerr_request
            .get_or_create(&labels)
            .set(request.status.as_f64());
    });
}

fn format_session_metrics(
    kind: &str,
    sessions: HashMap<String, Vec<Session>>,
    users: Vec<User>,
    registry: &mut Registry,
) {
    debug!("Formatting {sessions:?} as Prometheus");
    let sessions_labels = Family::<SessionLabels, Gauge<f64, AtomicU64>>::default();
    let sessions_percentage = Family::<SessionLabels, Gauge<f64, AtomicU64>>::default();
    let session_bandwidth = Family::<SessionBandwidth, Gauge<f64, AtomicU64>>::default();
    match kind {
        "plex" => {
            registry.register(
                "plex_sessions",
                format!("Plex sessions status"),
                sessions_labels.clone(),
            );
            registry.register(
                "plex_sessions_percentage",
                format!("Plex sessions percentage status"),
                sessions_percentage.clone(),
            );
            registry.register(
                "plex_session_bandwidth",
                format!("Plex session bandwidth"),
                session_bandwidth.clone(),
            );
        }
        "jellyfin" => {
            registry.register(
                "jellyfin_sessions",
                format!("Jellyfin sessions status"),
                sessions_labels.clone(),
            );
            registry.register(
                "jellyfin_sessions_percentage",
                format!("Jellyfin sessions percentage status"),
                sessions_percentage.clone(),
            );
        }
        _ => {
            registry.register(
                "sessions",
                format!("Sessions status"),
                sessions_labels.clone(),
            );
            registry.register(
                "sessions_percentage",
                format!("Sessions percentage status"),
                sessions_percentage.clone(),
            );
            registry.register(
                "session_bandwidth",
                format!("Session bandwidth"),
                session_bandwidth.clone(),
            );
        }
    }
    let mut wan_bandwidth = 0.0;
    let mut lan_bandwidth = 0.0;
    let mut inactive_users = users;
    sessions.into_iter().for_each(|(name, sessions)| {
        sessions.into_iter().for_each(|session| {
            match session.bandwidth.location {
                BandwidthLocation::Wan => wan_bandwidth += session.bandwidth.bandwidth as f64,
                BandwidthLocation::Lan => lan_bandwidth += session.bandwidth.bandwidth as f64,
                BandwidthLocation::Unknown => {}
            };
            inactive_users.retain(|user| user.name != session.user);
            let session_labels = SessionLabels {
                name: name.clone(),
                title: session.title,
                user: session.user,
                decision: session.stream_decision.to_string(),
                state: session.state,
                platform: session.platform,
                local: session.local as i8,
                relayed: session.relayed as i8,
                secure: session.secure as i8,
                address: session.address,
                public_address: session.location.ip_address,
                season_number: session.season_number,
                episode_number: session.episode_number,
                media_type: session.media_type,
                quality: session.quality,
                city: session.location.city,
                longitude: session.location.longitude,
                latitude: session.location.latitude,
            };

            sessions_percentage
                .get_or_create(&session_labels)
                .set(session.progress as f64);
            sessions_labels.get_or_create(&session_labels).set(1.0);
        });
        inactive_users.iter().for_each(|user| {
            sessions_labels
                .get_or_create(&SessionLabels {
                    name: name.to_string(),
                    title: "".to_string(),
                    user: user.name.clone(),
                    decision: "".to_string(),
                    state: "inactive".to_string(),
                    platform: "".to_string(),
                    local: 0,
                    relayed: 0,
                    secure: 0,
                    address: "".to_string(),
                    media_type: "".to_string(),
                    public_address: "".to_string(),
                    season_number: None,
                    episode_number: None,
                    quality: "".to_string(),
                    city: "".to_string(),
                    longitude: "".to_string(),
                    latitude: "".to_string(),
                })
                .set(0.0);
        });
        match kind {
            "plex" => {
                session_bandwidth
                    .get_or_create(&SessionBandwidth {
                        name: name.clone(),
                        location: "LAN".to_string(),
                    })
                    .set(lan_bandwidth);
                session_bandwidth
                    .get_or_create(&SessionBandwidth {
                        name,
                        location: "WAN".to_string(),
                    })
                    .set(wan_bandwidth);
            }
            _ => {}
        }
    });
}
fn format_library_metrics(
    kind: &str,
    libraries: HashMap<String, Vec<LibraryCount>>,
    registry: &mut Registry,
) {
    debug!("Formatting {libraries:?} as Prometheus");
    let movie_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let show_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let season_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let episode_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let show_library_label = Family::<PlexShowLabels, Gauge<f64, AtomicU64>>::default();
    let library_label = Family::<PlexLibraryLabels, Gauge<f64, AtomicU64>>::default();
    match kind {
        "plex" => {
            registry.register(
                "plex_movie_count",
                "Plex movie count",
                movie_count_label.clone(),
            );
            registry.register(
                "plex_show_count",
                "Plex show count",
                show_count_label.clone(),
            );
            registry.register(
                "plex_season_count",
                "Plex season count",
                season_count_label.clone(),
            );
            registry.register(
                "plex_episode_count",
                "Plex episode count",
                episode_count_label.clone(),
            );
            registry.register(
                "plex_show_library",
                "Plex show library",
                show_library_label.clone(),
            );
            registry.register("plex_library", "Plex library", library_label.clone());
        }
        "jellyfin" => {
            registry.register(
                "jellyfin_movie_count",
                "Jellyfin movie count",
                movie_count_label.clone(),
            );
            registry.register(
                "jellyfin_show_count",
                "Jellyfin show count",
                show_count_label.clone(),
            );
            registry.register(
                "jellyfin_episode_count",
                "Jellyfin episode count",
                episode_count_label.clone(),
            );
        }
        _ => {}
    }
    let mut movie_count = 0;
    let mut episode_count = 0;
    let mut season_count = 0;
    let mut show_count = 0;
    libraries.into_iter().for_each(|(name, library)| {
        library.into_iter().for_each(|lib| {
            let library_labels = PlexLibraryLabels {
                name: name.clone(),
                library_name: lib.name.clone(),
                library_type: lib.media_type.to_string(),
            };
            match lib.media_type {
                LibraryMediaType::Movie => {
                    movie_count += lib.count;
                    library_label
                        .get_or_create(&library_labels)
                        .set(lib.count as f64);
                }
                LibraryMediaType::Show => {
                    show_library_label
                        .get_or_create(&PlexShowLabels {
                            name: name.clone(),
                            library_name: lib.name.clone(),
                            library_type: lib.media_type.to_string(),
                            season_count: lib.child_count,
                            episode_count: lib.grand_child_count,
                        })
                        .set(lib.count as f64);
                    episode_count += lib.grand_child_count.unwrap_or(0);
                    season_count += lib.child_count.unwrap_or(0);
                    show_count += lib.count
                }
                _ => {
                    library_label
                        .get_or_create(&library_labels)
                        .set(lib.count as f64);
                }
            };
        });
    });
    movie_count_label
        .get_or_create(&EmptyLabel {})
        .set(movie_count as f64);
    show_count_label
        .get_or_create(&EmptyLabel {})
        .set(show_count as f64);
    season_count_label
        .get_or_create(&EmptyLabel {})
        .set(season_count as f64);
    episode_count_label
        .get_or_create(&EmptyLabel {})
        .set(episode_count as f64);
}
