use log::debug;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use crate::providers::overseerr::OverseerrRequest;
use crate::providers::plex::{LibraryInfos, PlexSessions};
use crate::providers::radarr::RadarrMovie;
use crate::providers::sonarr::SonarrEpisode;
use crate::providers::structs::tautulli::Library;
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
    PlexSession(HashMap<String, Vec<PlexSessions>>),
    PlexLibrary(HashMap<String, Vec<LibraryInfos>>),
    Default,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexSessionBandwidth {
    pub user: String,
    pub location: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexSessionLabels {
    pub name: String,
    pub title: String,
    pub user: String,
    pub decision: String,
    pub state: String,
    pub platform: String,
    pub local: i8,
    pub relayed: i8,
    pub media_type: String,
    pub secure: i8,
    pub address: String,
    pub public_address: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub quality: String,
    pub city: String,
    pub longitude: String,
    pub latitude: String,
}
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexSessionPercentageLabels {
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
struct PlexLibraryLabels {
    pub name: String,
    pub library_name: String,
    pub library_type: String,
    pub season_count: Option<i64>,
    pub episode_count: Option<i64>,
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
            TaskResult::Overseerr(overseerr) => format_overseerr_metrics(overseerr, &mut registry),
            TaskResult::PlexSession(sessions) => {
                format_plex_session_metrics(sessions, &mut registry)
            }
            TaskResult::PlexLibrary(libraries) => {
                format_plex_library_metrics(libraries, &mut registry)
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
    let tautulli_total_session = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
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
    let total_sessions = sessions.len();
    let labels = EmptyLabel {};
    tautulli_total_session
        .get_or_create(&labels)
        .inc_by(total_sessions as f64);
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

fn format_overseerr_metrics(requests: Vec<OverseerrRequest>, registry: &mut Registry) {
    debug!("Formatting {requests:?} as Prometheus");
    let overseerr_request = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();
    //let mut registy_request = HashMap::new();
    //let mut registy_media = HashMap::new();
    registry.register(
        "overseerr_requests",
        format!("overseerr requests status"),
        overseerr_request.clone(),
    );

    /*
    overseerr::MediaStatus::get_all()
        .into_iter()
        .for_each(|status| {
            registy_media.insert(
                status.to_string(),
                Family::<OverseerrRequestsLabels, Gauge<f64, AtomicU64>>::default(),
            );
            registry.register(
                &format!("overseerr_requests_{}", status.to_string()),
                format!("{}", status.to_description()),
                registy_media.get(&status.to_string()).unwrap().clone(),
            );
        });
    overseerr::RequestStatus::get_all()
        .into_iter()
        .for_each(|status| {
            registy_request.insert(
                status.to_string(),
                Family::<OverseerrRequestsLabels, Gauge<f64, AtomicU64>>::default(),
            );
            registry.register(
                &format!("overseerr_requests_{}", status.to_string()),
                format!("{}", status.to_description()),
                registy_request.get(&status.to_string()).unwrap().clone(),
            );
        });
    */
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
        /*match request.status.into() {
            overseerr::RequestStatus::Pending => {
                registy_request
                    .get(&overseerr::RequestStatus::Pending.to_string())
                    .unwrap()
                    .get_or_create(&OverseerrRequestsLabels {
                        kind: overseerr::RequestStatus::Pending.to_string(),
                    })
                    .inc();
            }
            overseerr::RequestStatus::Approved => {
                registy_request
                    .get(&overseerr::RequestStatus::Approved.to_string())
                    .unwrap()
                    .get_or_create(&OverseerrRequestsLabels {
                        kind: overseerr::RequestStatus::Approved.to_string(),
                    })
                    .inc();
            }
            overseerr::RequestStatus::Declined => {
                registy_request
                    .get(&overseerr::RequestStatus::Declined.to_string())
                    .unwrap()
                    .get_or_create(&OverseerrRequestsLabels {
                        kind: overseerr::RequestStatus::Declined.to_string(),
                    })
                    .inc();
            }
        };*/
    });
}

fn format_plex_session_metrics(
    sessions: HashMap<String, Vec<PlexSessions>>,
    registry: &mut Registry,
) {
    debug!("Formatting {sessions:?} as Prometheus");
    let plex_sessions = Family::<PlexSessionLabels, Gauge<f64, AtomicU64>>::default();
    let plex_sessions_percentage =
        Family::<PlexSessionPercentageLabels, Gauge<f64, AtomicU64>>::default();
    let plex_session_bandwidth = Family::<PlexSessionBandwidth, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "plex_sessions",
        format!("Plex sessions status"),
        plex_sessions.clone(),
    );
    registry.register(
        "plex_sessions_percentage",
        format!("Plex sessions percentage status"),
        plex_sessions_percentage.clone(),
    );
    registry.register(
        "plex_session_bandwidth",
        format!("Plex session bandwidth"),
        plex_session_bandwidth.clone(),
    );

    sessions.into_iter().for_each(|(name, sessions)| {
        sessions.into_iter().for_each(|session| {
            plex_sessions_percentage
                .get_or_create(&PlexSessionPercentageLabels {
                    name: name.clone(),
                    title: session.title.clone(),
                    user: session.user.clone(),
                    decision: session.stream_decision.to_string().clone(),
                    state: session.state.clone(),
                    platform: session.platform.clone(),
                    local: session.local as i8,
                    relayed: session.relayed as i8,
                    secure: session.secure as i8,
                    address: session.address.clone(),
                    public_address: session.location.ip_address.clone(),
                    season_number: session.season_number.clone(),
                    episode_number: session.episode_number.clone(),
                    media_type: session.media_type.clone(),
                    quality: session.quality.clone(),
                    city: session.location.city.clone(),
                    longitude: session.location.longitude.clone(),
                    latitude: session.location.latitude.clone(),
                })
                .set(session.progress as f64);
            plex_sessions
                .get_or_create(&PlexSessionLabels {
                    name: name.clone(),
                    title: session.title.clone(),
                    user: session.user.clone(),
                    decision: session.stream_decision.to_string().clone(),
                    state: session.state.clone(),
                    platform: session.platform.clone(),
                    local: session.local as i8,
                    relayed: session.relayed as i8,
                    secure: session.secure as i8,
                    address: session.address.clone(),
                    media_type: session.media_type.clone(),
                    public_address: session.location.ip_address.clone(),
                    season_number: session.season_number.clone(),
                    episode_number: session.episode_number.clone(),
                    quality: session.quality.clone(),
                    city: session.location.city.clone(),
                    longitude: session.location.longitude.clone(),
                    latitude: session.location.latitude.clone(),
                })
                .set(1.0);
            plex_session_bandwidth
                .get_or_create(&PlexSessionBandwidth {
                    user: session.user.clone(),
                    location: session.bandwidth.location.to_string(),
                })
                .set(session.bandwidth.bandwidth as f64);
        });
    });
}
fn format_plex_library_metrics(
    libraries: HashMap<String, Vec<LibraryInfos>>,
    registry: &mut Registry,
) {
    debug!("Formatting {libraries:?} as Prometheus");
    let plex_movie_count = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let plex_show_count = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let plex_season_count = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let plex_episode_count = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
    let plex_library = Family::<PlexLibraryLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "plex_library",
        format!("Plex library status"),
        plex_library.clone(),
    );
    registry.register(
        "plex_movie_count",
        format!("Plex movie count"),
        plex_movie_count.clone(),
    );
    registry.register(
        "plex_show_count",
        format!("Plex show count"),
        plex_show_count.clone(),
    );
    registry.register(
        "plex_season_count",
        format!("Plex season count"),
        plex_season_count.clone(),
    );
    registry.register(
        "plex_episode_count",
        format!("Plex episode count"),
        plex_episode_count.clone(),
    );
    let mut movie_count = 0;
    let mut episode_count = 0;
    let mut season_count = 0;
    let mut show_count = 0;
    libraries.into_iter().for_each(|(name, library)| {
        library.into_iter().for_each(|lib| {
            plex_library
                .get_or_create(&PlexLibraryLabels {
                    name: name.clone(),
                    library_name: lib.library_name.clone(),
                    library_type: lib.library_type.clone(),
                    season_count: lib.library_child_size,
                    episode_count: lib.library_grand_child_size,
                })
                .set(lib.library_size as f64);
            match lib.library_type.as_str() {
                "movie" => movie_count += lib.library_size,
                "show" => {
                    episode_count += lib.library_grand_child_size.unwrap_or(0);
                    season_count += lib.library_child_size.unwrap_or(0);
                    show_count += lib.library_size
                }
                _ => {}
            }
        });
    });
    plex_movie_count
        .get_or_create(&EmptyLabel {})
        .set(movie_count as f64);
    plex_show_count
        .get_or_create(&EmptyLabel {})
        .set(show_count as f64);
    plex_season_count
        .get_or_create(&EmptyLabel {})
        .set(season_count as f64);
    plex_episode_count
        .get_or_create(&EmptyLabel {})
        .set(episode_count as f64);
}
