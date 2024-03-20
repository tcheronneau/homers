use log::debug;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;
use anyhow::Result;

use crate::providers::sonarr::SonarrEpisode;
use crate::providers::tautulli::SessionSummary;
use crate::providers::radarr::RadarrMovie;
use crate::providers::structs::tautulli::Library;
use crate::providers::overseerr::OverseerrRequest;


#[derive(PartialEq, Debug, Eq, Copy, Clone)]
pub enum Format {
    Prometheus,
    OpenMetrics,
}


pub enum TaskResult {
    SonarrToday(Result<Vec<SonarrEpisode>>),
    SonarrMissing(Result<Vec<SonarrEpisode>>),
    TautulliSessionPercentage(Result<Vec<SessionSummary>>),
    TautulliSession(Result<Vec<SessionSummary>>),
    TautulliLibrary(Result<Vec<Library>>),
    Radarr(Result<Vec<RadarrMovie>>),
    Overseerr(Result<Vec<OverseerrRequest>>),
    Default,
}


#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SonarrLabels {
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
    pub media_title: String,
    pub requested_at: String,
}

pub fn format_metrics(task_result: Vec<TaskResult>) -> anyhow::Result<String> {
    let mut buffer = String::new();
    let mut registry = Registry::with_prefix("homers");
    for task_result in task_result {
        match task_result {
            TaskResult::SonarrToday(episodes) => format_sonarr_today_metrics(episodes?, &mut registry),
            TaskResult::SonarrMissing(episodes) => format_sonarr_missing_metrics(episodes?, &mut registry),
            TaskResult::TautulliSessionPercentage(sessions) => format_tautulli_session_percentage_metrics(sessions?, &mut registry),
            TaskResult::TautulliSession(sessions) => format_tautulli_session_metrics(sessions?, &mut registry),
            TaskResult::TautulliLibrary(libraries) => format_tautulli_library_metrics(libraries?, &mut registry),
            TaskResult::Radarr(movies) => format_radarr_metrics(movies?, &mut registry),
            TaskResult::Overseerr(overseerr) => format_overseerr_metrics(overseerr?, &mut registry),
            TaskResult::Default => return Err(anyhow::anyhow!("No task result")),
        }
    }
    encode(&mut buffer, &registry)?;
    Ok(buffer)
}

pub fn format_sonarr_today_metrics(episodes: Vec<SonarrEpisode>, registry: &mut Registry) {
    debug!("Formatting {episodes:?} as Prometheus");
    let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "sonarr_today_episode",
        format!("Sonarr today episode status"),
        sonarr_episode.clone(),
    );
    for episode in episodes {
        let labels = SonarrLabels {
            sxe: episode.sxe.clone(),
            season_number: episode.season_number,
            episode_number: episode.episode_number,
            title: episode.title.clone(),
            serie: episode.serie.clone(),
        };
        sonarr_episode 
            .get_or_create(&labels)
            .set(if episode.has_file { 1.0 } else { 0.0 });
    }
}
pub fn format_sonarr_missing_metrics(episodes: Vec<SonarrEpisode>, registry: &mut Registry) {
    debug!("Formatting {episodes:?} as Prometheus");
    let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "sonarr_missing_episode",
        format!("Sonarr missing episode status"),
        sonarr_episode.clone(),
    );
    for episode in episodes {
        let labels = SonarrLabels {
            sxe: episode.sxe.clone(),
            season_number: episode.season_number,
            episode_number: episode.episode_number,
            title: episode.title.clone(),
            serie: episode.serie.clone(),
        };
        sonarr_episode 
            .get_or_create(&labels)
            .set(if episode.has_file { 1.0 } else { 0.0 });
    }
}
pub fn format_tautulli_session_percentage_metrics(sessions: Vec<SessionSummary>, registry: &mut Registry) { 
    debug!("Formatting {sessions:?} as Prometheus");
    let tautulli_session = Family::<TautulliSessionPercentageLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "tautulli_session_percentage",
        format!("Tautulli session progress"),
        tautulli_session.clone(),
    );
    for session in sessions {
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
        tautulli_session 
            .get_or_create(&labels)
            .set(session.progress.parse::<f64>().unwrap());
    }
}
pub fn format_tautulli_session_metrics(sessions: Vec<SessionSummary>, registry: &mut Registry) { 
    debug!("Formatting {sessions:?} as Prometheus");
    let tautulli_session = Family::<TautulliSessionLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "tautulli_session",
        format!("Tautulli session status"),
        tautulli_session.clone(),
    );
    for session in sessions {
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
        tautulli_session 
            .get_or_create(&labels)
            .set(1.0);
    }
}
pub fn format_tautulli_library_metrics(libraries: Vec<Library>, registry: &mut Registry) { 
    debug!("Formatting {libraries:?} as Prometheus");
    let tautulli_library = Family::<TautulliLibraryLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "tautulli_library",
        format!("Tautulli library status"),
        tautulli_library.clone(),
    );
    for library in libraries {
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
    }
}

pub fn format_radarr_metrics(movies: Vec<RadarrMovie>, registry: &mut Registry) {
    debug!("Formatting {movies:?} as Prometheus");
    let radarr_movie = Family::<RadarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "radarr_movie",
        format!("Radarr movie status"),
        radarr_movie.clone(),
    );
    for movie in movies {
        let labels = RadarrLabels {
            title: movie.title.clone(),
            is_available: movie.is_available as i8,
            monitored: movie.monitored as i8,
            missing_available: movie.missing_available as i8,
        };
        radarr_movie 
            .get_or_create(&labels)
            .set(if movie.has_file { 1.0 } else { 0.0 });
    }
}
pub fn format_overseerr_metrics(requests: Vec<OverseerrRequest>, registry: &mut Registry) {
    debug!("Formatting {requests:?} as Prometheus");
    let overseerr_request = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "overseerr_requests",
        format!("overseerr requests status"),
        overseerr_request.clone(),
    );
    for request in requests {
        let labels = OverseerrLabels {
            media_type: request.media_type.clone(),
            requested_by: request.requested_by.to_string(),
            request_status: request.status.to_string(),
            media_title: request.media_title,
            requested_at: request.requested_at,
        };
        overseerr_request
            .get_or_create(&labels)
            .set(request.media_status as f64);
    }
}
