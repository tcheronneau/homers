use log::debug;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;

use crate::providers::overseerr::OverseerrRequest;
use crate::providers::radarr::RadarrMovie;
use crate::providers::sonarr::SonarrEpisode;
use crate::providers::structs::{
    BandwidthLocation, LibraryCount, MediaType as LibraryMediaType, Session,
};
use crate::providers::tautulli::Library as TautulliLibrary;
use crate::providers::tautulli::SessionSummary;
use crate::tasks::{
    LibraryResult, OverseerrRequestResult, RadarrMovieResult, SessionResult, SonarrEpisodeResult,
    SonarrMissingResult, TaskResult, TautulliLibraryResult, TautulliSessionResult,
};

#[derive(PartialEq, Debug, Eq, Copy, Clone)]
pub enum Format {
    Prometheus,
    OpenMetrics,
}

pub trait FormatAsPrometheus {
    fn format_as_prometheus(&self, registry: &mut Registry);
}

// --- Label structs ---

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct EmptyLabel {}

// Session labels (Plex/Jellyfin)
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionInfoLabels {
    pub name: String,
    pub user: String,
    pub title: String,
    pub state: String,
    pub platform: String,
    pub decision: String,
    pub media_type: String,
    pub quality: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionProgressLabels {
    pub name: String,
    pub user: String,
    pub title: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionCountLabels {
    pub name: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct UserActiveLabels {
    pub name: String,
    pub user: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionBandwidthLabels {
    pub name: String,
    pub location: String,
}

// Tautulli session labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliSessionInfoLabels {
    pub user: String,
    pub title: String,
    pub state: String,
    pub media_type: String,
    pub quality: String,
    pub quality_profile: String,
    pub video_stream: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliSessionProgressLabels {
    pub user: String,
    pub title: String,
}

// Tautulli library labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliLibraryLabels {
    pub section_name: String,
    pub section_type: String,
}

// Radarr labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct RadarrLabels {
    pub name: String,
    pub title: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct RadarrAggregateLabels {
    pub name: String,
}

// Sonarr labels (sxe field removed)
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SonarrLabels {
    pub name: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub serie: String,
}

// Overseerr labels (requested_at removed, split into two metrics)
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct OverseerrLabels {
    pub media_type: String,
    pub requested_by: String,
    pub media_title: String,
}

// Plex/Jellyfin library labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct PlexLibraryLabels {
    pub name: String,
    pub library_name: String,
    pub library_type: String,
}

pub fn format_metrics(task_result: Vec<TaskResult>) -> anyhow::Result<String> {
    let mut buffer = String::new();
    let mut registry = Registry::with_prefix("homers");
    for task_result in task_result {
        task_result.format_as_prometheus(&mut registry);
    }
    encode(&mut buffer, &registry)?;
    Ok(buffer)
}

// --- FormatAsPrometheus implementations ---

impl FormatAsPrometheus for SonarrEpisodeResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
        registry.register(
            "sonarr_today_episode",
            "Sonarr today episode status".to_string(),
            sonarr_episode.clone(),
        );
        self.episodes.iter().for_each(|ep: &SonarrEpisode| {
            let labels = SonarrLabels {
                name: self.name.clone(),
                season_number: ep.season_number,
                episode_number: ep.episode_number,
                title: ep.title.clone(),
                serie: ep.serie.clone(),
            };
            sonarr_episode
                .get_or_create(&labels)
                .set(if ep.has_file { 1.0 } else { 0.0 });
        });
    }
}

impl FormatAsPrometheus for SonarrMissingResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
        registry.register(
            "sonarr_missing_episode",
            "Sonarr missing episode status".to_string(),
            sonarr_episode.clone(),
        );
        self.episodes.iter().for_each(|ep: &SonarrEpisode| {
            let labels = SonarrLabels {
                name: self.name.clone(),
                season_number: ep.season_number,
                episode_number: ep.episode_number,
                title: ep.title.clone(),
                serie: ep.serie.clone(),
            };
            sonarr_episode
                .get_or_create(&labels)
                .set(if ep.has_file { 1.0 } else { 0.0 });
        });
    }
}

impl FormatAsPrometheus for TautulliSessionResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let session_count = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
        let session_info = Family::<TautulliSessionInfoLabels, Gauge<f64, AtomicU64>>::default();
        let session_progress =
            Family::<TautulliSessionProgressLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "tautulli_session_count",
            "Tautulli active session count".to_string(),
            session_count.clone(),
        );
        registry.register(
            "tautulli_session_info",
            "Tautulli session info".to_string(),
            session_info.clone(),
        );
        registry.register(
            "tautulli_session_progress",
            "Tautulli session progress percentage".to_string(),
            session_progress.clone(),
        );

        session_count
            .get_or_create(&EmptyLabel {})
            .set(self.sessions.len() as f64);

        self.sessions.iter().for_each(|session: &SessionSummary| {
            let info_labels = TautulliSessionInfoLabels {
                user: session.user.clone(),
                title: session.title.clone(),
                state: session.state.clone(),
                media_type: session.media_type.clone(),
                quality: session.quality.clone(),
                quality_profile: session.quality_profile.clone(),
                video_stream: session.video_stream.clone(),
            };
            session_info.get_or_create(&info_labels).set(1.0);

            let progress_labels = TautulliSessionProgressLabels {
                user: session.user.clone(),
                title: session.title.clone(),
            };
            session_progress
                .get_or_create(&progress_labels)
                .set(session.progress.parse::<f64>().unwrap_or(0.0));
        });
    }
}

impl FormatAsPrometheus for TautulliLibraryResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let item_count = Family::<TautulliLibraryLabels, Gauge<f64, AtomicU64>>::default();
        let parent_count = Family::<TautulliLibraryLabels, Gauge<f64, AtomicU64>>::default();
        let child_count = Family::<TautulliLibraryLabels, Gauge<f64, AtomicU64>>::default();
        let active = Family::<TautulliLibraryLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "tautulli_library_item_count",
            "Tautulli library item count".to_string(),
            item_count.clone(),
        );
        registry.register(
            "tautulli_library_parent_count",
            "Tautulli library parent count".to_string(),
            parent_count.clone(),
        );
        registry.register(
            "tautulli_library_child_count",
            "Tautulli library child count".to_string(),
            child_count.clone(),
        );
        registry.register(
            "tautulli_library_active",
            "Tautulli library active status".to_string(),
            active.clone(),
        );

        self.libraries.iter().for_each(|library: &TautulliLibrary| {
            let labels = TautulliLibraryLabels {
                section_name: library.section_name.clone(),
                section_type: library.section_type.clone(),
            };
            item_count
                .get_or_create(&labels)
                .set(library.count.parse::<f64>().unwrap_or(0.0));
            parent_count
                .get_or_create(&labels)
                .set(parse_optional_count(&library.parent_count));
            child_count
                .get_or_create(&labels)
                .set(parse_optional_count(&library.child_count));
            active
                .get_or_create(&labels)
                .set(library.is_active as f64);
        });
    }
}

fn parse_optional_count(value: &Option<String>) -> f64 {
    value
        .as_ref()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0)
}

impl FormatAsPrometheus for RadarrMovieResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let has_file = Family::<RadarrLabels, Gauge<f64, AtomicU64>>::default();
        let monitored = Family::<RadarrLabels, Gauge<f64, AtomicU64>>::default();
        let available = Family::<RadarrLabels, Gauge<f64, AtomicU64>>::default();
        let movies_total = Family::<RadarrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let monitored_total = Family::<RadarrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let missing_total = Family::<RadarrAggregateLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "radarr_movie_has_file",
            "Radarr movie has file on disk".to_string(),
            has_file.clone(),
        );
        registry.register(
            "radarr_movie_monitored",
            "Radarr movie is monitored".to_string(),
            monitored.clone(),
        );
        registry.register(
            "radarr_movie_available",
            "Radarr movie is available".to_string(),
            available.clone(),
        );
        registry.register(
            "radarr_movies_total",
            "Radarr total movie count".to_string(),
            movies_total.clone(),
        );
        registry.register(
            "radarr_movies_monitored_total",
            "Radarr monitored movie count".to_string(),
            monitored_total.clone(),
        );
        registry.register(
            "radarr_movies_missing_total",
            "Radarr missing available movie count".to_string(),
            missing_total.clone(),
        );

        let agg_labels = RadarrAggregateLabels {
            name: self.name.clone(),
        };
        let mut total = 0_f64;
        let mut mon_total = 0_f64;
        let mut miss_total = 0_f64;

        self.movies.iter().for_each(|movie: &RadarrMovie| {
            let labels = RadarrLabels {
                name: self.name.clone(),
                title: movie.title.clone(),
            };
            has_file
                .get_or_create(&labels)
                .set(if movie.has_file { 1.0 } else { 0.0 });
            monitored
                .get_or_create(&labels)
                .set(if movie.monitored { 1.0 } else { 0.0 });
            available
                .get_or_create(&labels)
                .set(if movie.is_available { 1.0 } else { 0.0 });

            total += 1.0;
            if movie.monitored {
                mon_total += 1.0;
            }
            if movie.missing_available {
                miss_total += 1.0;
            }
        });

        movies_total.get_or_create(&agg_labels).set(total);
        monitored_total.get_or_create(&agg_labels).set(mon_total);
        missing_total.get_or_create(&agg_labels).set(miss_total);
    }
}

impl FormatAsPrometheus for OverseerrRequestResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let request_status = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();
        let media_status = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            format!("{}_request_status", self.kind),
            format!("{} request status", self.kind),
            request_status.clone(),
        );
        registry.register(
            format!("{}_media_status", self.kind),
            format!("{} media status", self.kind),
            media_status.clone(),
        );

        self.requests.iter().for_each(|request: &OverseerrRequest| {
            let labels = OverseerrLabels {
                media_type: request.media_type.clone(),
                requested_by: request.requested_by.to_string(),
                media_title: request.media_title.clone(),
            };
            request_status
                .get_or_create(&labels)
                .set(request.status.as_f64());
            media_status
                .get_or_create(&labels)
                .set(request.media_status.as_f64());
        });
    }
}

impl FormatAsPrometheus for SessionResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let session_count = Family::<SessionCountLabels, Gauge<f64, AtomicU64>>::default();
        let session_info = Family::<SessionInfoLabels, Gauge<f64, AtomicU64>>::default();
        let session_progress = Family::<SessionProgressLabels, Gauge<f64, AtomicU64>>::default();
        let user_active = Family::<UserActiveLabels, Gauge<f64, AtomicU64>>::default();
        let session_bandwidth = Family::<SessionBandwidthLabels, Gauge<f64, AtomicU64>>::default();

        let mut inactive_users = self.users.clone();
        let mut wan_bandwidth = 0.0;
        let mut lan_bandwidth = 0.0;

        let kind = self.kind.as_str();
        let prefix = match kind {
            "plex" => "plex",
            "jellyfin" => "jellyfin",
            _ => "unknown",
        };

        registry.register(
            format!("{prefix}_session_count"),
            format!("{prefix} active session count"),
            session_count.clone(),
        );
        registry.register(
            format!("{prefix}_session_info"),
            format!("{prefix} session info"),
            session_info.clone(),
        );
        registry.register(
            format!("{prefix}_session_progress"),
            format!("{prefix} session progress percentage"),
            session_progress.clone(),
        );
        registry.register(
            format!("{prefix}_user_active"),
            format!("{prefix} user active status"),
            user_active.clone(),
        );
        if kind == "plex" || kind != "jellyfin" {
            registry.register(
                format!("{prefix}_session_bandwidth"),
                format!("{prefix} session bandwidth"),
                session_bandwidth.clone(),
            );
        }

        self.sessions.iter().for_each(|session: &Session| {
            match session.bandwidth.location {
                BandwidthLocation::Wan => wan_bandwidth += session.bandwidth.bandwidth as f64,
                BandwidthLocation::Lan => lan_bandwidth += session.bandwidth.bandwidth as f64,
                BandwidthLocation::Unknown => {}
            };
            inactive_users.retain(|u| u.name != session.user);

            let info_labels = SessionInfoLabels {
                name: self.name.clone(),
                user: session.user.clone(),
                title: session.title.clone(),
                state: session.state.to_string(),
                platform: session.platform.to_string(),
                decision: session.stream_decision.to_string(),
                media_type: session.media_type.to_string(),
                quality: session.quality.to_string(),
            };
            session_info.get_or_create(&info_labels).set(1.0);

            let progress_labels = SessionProgressLabels {
                name: self.name.clone(),
                user: session.user.clone(),
                title: session.title.clone(),
            };
            session_progress
                .get_or_create(&progress_labels)
                .set(session.progress);

            user_active
                .get_or_create(&UserActiveLabels {
                    name: self.name.clone(),
                    user: session.user.clone(),
                })
                .set(1.0);
        });

        // Set session count
        session_count
            .get_or_create(&SessionCountLabels {
                name: self.name.clone(),
            })
            .set(self.sessions.len() as f64);

        // Set bandwidth AFTER the loop (fixes bug where values were set before accumulation)
        if kind == "plex" || kind != "jellyfin" {
            session_bandwidth
                .get_or_create(&SessionBandwidthLabels {
                    name: self.name.clone(),
                    location: "LAN".to_string(),
                })
                .set(lan_bandwidth);
            session_bandwidth
                .get_or_create(&SessionBandwidthLabels {
                    name: self.name.clone(),
                    location: "WAN".to_string(),
                })
                .set(wan_bandwidth);
        }

        // Mark inactive users
        inactive_users.iter().for_each(|u| {
            user_active
                .get_or_create(&UserActiveLabels {
                    name: self.name.clone(),
                    user: u.name.clone(),
                })
                .set(0.0);
        });
    }
}

impl FormatAsPrometheus for LibraryResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let movie_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
        let show_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
        let season_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
        let episode_count_label = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
        let library_count = Family::<PlexLibraryLabels, Gauge<f64, AtomicU64>>::default();
        let library_child_count = Family::<PlexLibraryLabels, Gauge<f64, AtomicU64>>::default();
        let library_grandchild_count =
            Family::<PlexLibraryLabels, Gauge<f64, AtomicU64>>::default();

        let mut movie_count = 0;
        let mut episode_count = 0;
        let mut season_count = 0;
        let mut show_count = 0;

        match self.kind.as_str() {
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
                    "plex_library_count",
                    "Plex library item count",
                    library_count.clone(),
                );
                registry.register(
                    "plex_library_child_count",
                    "Plex library child count (seasons)",
                    library_child_count.clone(),
                );
                registry.register(
                    "plex_library_grandchild_count",
                    "Plex library grandchild count (episodes)",
                    library_grandchild_count.clone(),
                );
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
                registry.register(
                    "jellyfin_library_count",
                    "Jellyfin library item count",
                    library_count.clone(),
                );
                registry.register(
                    "jellyfin_library_child_count",
                    "Jellyfin library child count",
                    library_child_count.clone(),
                );
                registry.register(
                    "jellyfin_library_grandchild_count",
                    "Jellyfin library grandchild count",
                    library_grandchild_count.clone(),
                );
            }
            _ => {}
        }

        self.libraries.iter().for_each(|lib: &LibraryCount| {
            let labels = PlexLibraryLabels {
                name: self.name.clone(),
                library_name: lib.name.clone(),
                library_type: lib.media_type.to_string(),
            };
            library_count
                .get_or_create(&labels)
                .set(lib.count as f64);
            library_child_count
                .get_or_create(&labels)
                .set(lib.child_count.unwrap_or(0) as f64);
            library_grandchild_count
                .get_or_create(&labels)
                .set(lib.grand_child_count.unwrap_or(0) as f64);

            match lib.media_type {
                LibraryMediaType::Movie => {
                    movie_count += lib.count;
                }
                LibraryMediaType::Show => {
                    episode_count += lib.grand_child_count.unwrap_or(0);
                    season_count += lib.child_count.unwrap_or(0);
                    show_count += lib.count;
                }
                _ => {}
            };
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::sonarr::SonarrEpisode;
    use crate::tasks::{SonarrEpisodeResult, TaskResult};

    #[test]
    fn test_format_metrics() {
        let task_result = vec![TaskResult::SonarrToday(SonarrEpisodeResult {
            name: "test".to_string(),
            episodes: vec![SonarrEpisode {
                sxe: "S01E01".to_string(),
                season_number: 1,
                episode_number: 1,
                air_date: "2021-01-01".to_string(),
                title: "Test".to_string(),
                serie: "Test".to_string(),
                has_file: true,
            }],
        })];
        let result = format_metrics(task_result).unwrap();
        assert_eq!(
            result,
            "# HELP homers_sonarr_today_episode Sonarr today episode status.\n# TYPE homers_sonarr_today_episode gauge\nhomers_sonarr_today_episode{name=\"test\",season_number=\"1\",episode_number=\"1\",title=\"Test\",serie=\"Test\"} 1.0\n# EOF\n"
        );
    }
}
