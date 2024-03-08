use log::debug;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;

use crate::providers::sonarr::SonarrEpisode;
use crate::providers::tautulli::SessionSummary;


#[derive(PartialEq, Debug, Eq, Copy, Clone)]
pub enum Format {
    Prometheus,
    OpenMetrics,
}

pub enum TaskResult {
    Sonarr(Vec<SonarrEpisode>),
    Tautulli(Vec<SessionSummary>),
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
struct TautulliLabels {
    pub user: String,
    pub title: String,
    pub state: String,
    pub media_type: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
}

pub fn format_metrics(task_result: Vec<TaskResult>) -> anyhow::Result<String> {
    let mut buffer = String::new();
    let mut registry = Registry::with_prefix("homers");
    for task_result in task_result {
        match task_result {
            TaskResult::Sonarr(episodes) => format_sonarr_metrics(episodes, &mut registry),
            TaskResult::Tautulli(sessions) => format_tautulli_metrics(sessions, &mut registry),
            TaskResult::Default => return Err(anyhow::anyhow!("No task result")),
        }
    }
    encode(&mut buffer, &registry)?;
    Ok(buffer)
}

pub fn format_sonarr_metrics(episodes: Vec<SonarrEpisode>, registry: &mut Registry) {
    debug!("Formatting {episodes:?} as Prometheus");
    let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "sonarr_episode",
        format!("Sonarr episode status"),
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
pub fn format_tautulli_metrics(sessions: Vec<SessionSummary>, registry: &mut Registry) { 
    debug!("Formatting {sessions:?} as Prometheus");
    let tautulli_session = Family::<TautulliLabels, Gauge<f64, AtomicU64>>::default();
    registry.register(
        "tautulli_session",
        format!("Tautulli session status"),
        tautulli_session.clone(),
    );
    for session in sessions {
        let labels = TautulliLabels {
            user: session.user.clone(),
            title: session.title.clone(),
            state: session.state.clone(),
            media_type: session.media_type.clone(),
            season_number: session.season_number.clone(),
            episode_number: session.episode_number.clone(),
        };
        tautulli_session 
            .get_or_create(&labels)
            .set(session.progress.parse::<f64>().unwrap());
    }
}
