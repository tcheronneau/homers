use log::debug;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;

use crate::providers::lidarr::LidarrArtist;
use crate::providers::overseerr::OverseerrRequest;
use crate::providers::radarr::RadarrMovie;
use crate::providers::readarr::ReadarrAuthor;
use crate::providers::sonarr::SonarrEpisode;
use crate::providers::structs::{
    BandwidthLocation, LibraryCount, MediaType as LibraryMediaType, Session,
};
use crate::providers::tautulli::{Library as TautulliLibrary, SessionSummary};
use crate::tasks::{
    LibraryResult, LidarrArtistResult, OverseerrRequestResult, RadarrMovieResult,
    ReadarrAuthorResult, SessionResult, SonarrEpisodeResult, SonarrMissingResult, TaskResult,
    TautulliHistoryResult, TautulliLibraryResult, TautulliSessionResult,
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

// Tautulli history labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliHistoryUserLabels {
    pub user: String,
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

// Lidarr labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct LidarrLabels {
    pub name: String,
    pub artist: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct LidarrAggregateLabels {
    pub name: String,
}

// Readarr labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct ReadarrLabels {
    pub name: String,
    pub author: String,
}

#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct ReadarrAggregateLabels {
    pub name: String,
}

// Sonarr labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SonarrLabels {
    pub name: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub serie: String,
    pub sxe: String,
}

// Session location labels (geo data)
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SessionLocationLabels {
    pub name: String,
    pub user: String,
    pub title: String,
    pub city: String,
    pub country: String,
    pub latitude: String,
    pub longitude: String,
}

// Tautulli session location labels (geo data)
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct TautulliSessionLocationLabels {
    pub user: String,
    pub title: String,
    pub city: String,
    pub country: String,
    pub latitude: String,
    pub longitude: String,
}

// Sonarr aggregate labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct SonarrAggregateLabels {
    pub name: String,
}

// Overseerr aggregate labels
#[derive(Clone, Hash, Eq, PartialEq, EncodeLabelSet, Debug)]
struct OverseerrAggregateLabels {
    pub name: String,
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
        let episodes_total = Family::<SonarrAggregateLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "sonarr_today_episode",
            "Sonarr today episode status".to_string(),
            sonarr_episode.clone(),
        );
        registry.register(
            "sonarr_today_episodes_total",
            "Sonarr today episodes count".to_string(),
            episodes_total.clone(),
        );

        self.episodes.iter().for_each(|ep: &SonarrEpisode| {
            let labels = SonarrLabels {
                name: self.name.clone(),
                season_number: ep.season_number,
                episode_number: ep.episode_number,
                title: ep.title.clone(),
                serie: ep.serie.clone(),
                sxe: ep.sxe.clone(),
            };
            sonarr_episode
                .get_or_create(&labels)
                .set(if ep.has_file { 1.0 } else { 0.0 });
        });

        episodes_total
            .get_or_create(&SonarrAggregateLabels {
                name: self.name.clone(),
            })
            .set(self.episodes.len() as f64);
    }
}

impl FormatAsPrometheus for SonarrMissingResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let sonarr_episode = Family::<SonarrLabels, Gauge<f64, AtomicU64>>::default();
        let episodes_total = Family::<SonarrAggregateLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "sonarr_missing_episode",
            "Sonarr missing episode status".to_string(),
            sonarr_episode.clone(),
        );
        registry.register(
            "sonarr_missing_episodes_total",
            "Sonarr missing episodes count".to_string(),
            episodes_total.clone(),
        );

        self.episodes.iter().for_each(|ep: &SonarrEpisode| {
            let labels = SonarrLabels {
                name: self.name.clone(),
                season_number: ep.season_number,
                episode_number: ep.episode_number,
                title: ep.title.clone(),
                serie: ep.serie.clone(),
                sxe: ep.sxe.clone(),
            };
            sonarr_episode
                .get_or_create(&labels)
                .set(if ep.has_file { 1.0 } else { 0.0 });
        });

        episodes_total
            .get_or_create(&SonarrAggregateLabels {
                name: self.name.clone(),
            })
            .set(self.episodes.len() as f64);
    }
}

impl FormatAsPrometheus for TautulliSessionResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let session_count = Family::<EmptyLabel, Gauge<f64, AtomicU64>>::default();
        let session_info = Family::<TautulliSessionInfoLabels, Gauge<f64, AtomicU64>>::default();
        let session_progress =
            Family::<TautulliSessionProgressLabels, Gauge<f64, AtomicU64>>::default();
        let session_location =
            Family::<TautulliSessionLocationLabels, Gauge<f64, AtomicU64>>::default();

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
        registry.register(
            "tautulli_session_location",
            "Tautulli session location".to_string(),
            session_location.clone(),
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

            let location_labels = TautulliSessionLocationLabels {
                user: session.user.clone(),
                title: session.title.clone(),
                city: session.location.city.clone(),
                country: session.location.country.clone(),
                latitude: session.location.latitude.clone(),
                longitude: session.location.longitude.clone(),
            };
            session_location.get_or_create(&location_labels).set(1.0);
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

impl FormatAsPrometheus for TautulliHistoryResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting TautulliHistoryResult as Prometheus");
        let total_plays = Gauge::<f64, AtomicU64>::default();
        let user_watches =
            Family::<TautulliHistoryUserLabels, Gauge<f64, AtomicU64>>::default();
        let plays_24h = Gauge::<f64, AtomicU64>::default();

        registry.register(
            "tautulli_history_total_plays",
            "Total number of plays in Tautulli history (all time)",
            total_plays.clone(),
        );
        registry.register(
            "tautulli_history_user_watches_24h",
            "Number of items watched per user in last 24 hours",
            user_watches.clone(),
        );
        registry.register(
            "tautulli_history_plays_24h",
            "Total number of plays in last 24 hours",
            plays_24h.clone(),
        );

        total_plays.set(self.total_plays as f64);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let twenty_four_hours_ago = now - 86400;

        let mut user_counts: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut total_24h = 0.0;

        for entry in &self.entries {
            if entry.date >= twenty_four_hours_ago {
                total_24h += 1.0;
                let user = if entry.friendly_name.is_empty() {
                    entry.user.clone()
                } else {
                    entry.friendly_name.clone()
                };
                *user_counts.entry(user).or_insert(0.0) += 1.0;
            }
        }

        plays_24h.set(total_24h);
        for (user, count) in &user_counts {
            user_watches
                .get_or_create(&TautulliHistoryUserLabels {
                    user: user.clone(),
                })
                .set(*count);
        }
    }
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

impl FormatAsPrometheus for LidarrArtistResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let monitored = Family::<LidarrLabels, Gauge<f64, AtomicU64>>::default();
        let artists_total = Family::<LidarrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let monitored_total = Family::<LidarrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let tracks_total = Family::<LidarrAggregateLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "lidarr_artist_monitored",
            "Lidarr artist is monitored".to_string(),
            monitored.clone(),
        );
        registry.register(
            "lidarr_artists_total",
            "Lidarr total artist count".to_string(),
            artists_total.clone(),
        );
        registry.register(
            "lidarr_monitored_artists_total",
            "Lidarr monitored artist count".to_string(),
            monitored_total.clone(),
        );
        registry.register(
            "lidarr_tracks_total",
            "Lidarr total track file count".to_string(),
            tracks_total.clone(),
        );

        let agg_labels = LidarrAggregateLabels {
            name: self.name.clone(),
        };
        let mut total = 0_f64;
        let mut mon_total = 0_f64;
        let mut track_total = 0_f64;

        self.artists.iter().for_each(|artist: &LidarrArtist| {
            let labels = LidarrLabels {
                name: self.name.clone(),
                artist: artist.name.clone(),
            };
            monitored
                .get_or_create(&labels)
                .set(if artist.monitored { 1.0 } else { 0.0 });

            total += 1.0;
            if artist.monitored {
                mon_total += 1.0;
            }
            track_total += artist.track_file_count as f64;
        });

        artists_total.get_or_create(&agg_labels).set(total);
        monitored_total.get_or_create(&agg_labels).set(mon_total);
        tracks_total.get_or_create(&agg_labels).set(track_total);
    }
}

impl FormatAsPrometheus for ReadarrAuthorResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let monitored = Family::<ReadarrLabels, Gauge<f64, AtomicU64>>::default();
        let authors_total = Family::<ReadarrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let monitored_total = Family::<ReadarrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let books_total = Family::<ReadarrAggregateLabels, Gauge<f64, AtomicU64>>::default();

        registry.register(
            "readarr_author_monitored",
            "Readarr author is monitored".to_string(),
            monitored.clone(),
        );
        registry.register(
            "readarr_authors_total",
            "Readarr total author count".to_string(),
            authors_total.clone(),
        );
        registry.register(
            "readarr_monitored_authors_total",
            "Readarr monitored author count".to_string(),
            monitored_total.clone(),
        );
        registry.register(
            "readarr_books_total",
            "Readarr total book file count".to_string(),
            books_total.clone(),
        );

        let agg_labels = ReadarrAggregateLabels {
            name: self.name.clone(),
        };
        let mut total = 0_f64;
        let mut mon_total = 0_f64;
        let mut book_total = 0_f64;

        self.authors.iter().for_each(|author: &ReadarrAuthor| {
            let labels = ReadarrLabels {
                name: self.name.clone(),
                author: author.name.clone(),
            };
            monitored
                .get_or_create(&labels)
                .set(if author.monitored { 1.0 } else { 0.0 });

            total += 1.0;
            if author.monitored {
                mon_total += 1.0;
            }
            book_total += author.book_file_count as f64;
        });

        authors_total.get_or_create(&agg_labels).set(total);
        monitored_total.get_or_create(&agg_labels).set(mon_total);
        books_total.get_or_create(&agg_labels).set(book_total);
    }
}

impl FormatAsPrometheus for OverseerrRequestResult {
    fn format_as_prometheus(&self, registry: &mut Registry) {
        debug!("Formatting {self:?} as Prometheus");
        let request_status = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();
        let media_status = Family::<OverseerrLabels, Gauge<f64, AtomicU64>>::default();
        let requests_total = Family::<OverseerrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let pending_total = Family::<OverseerrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let approved_total = Family::<OverseerrAggregateLabels, Gauge<f64, AtomicU64>>::default();
        let declined_total = Family::<OverseerrAggregateLabels, Gauge<f64, AtomicU64>>::default();

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
        registry.register(
            format!("{}_requests_total", self.kind),
            format!("{} total request count", self.kind),
            requests_total.clone(),
        );
        registry.register(
            format!("{}_requests_pending_total", self.kind),
            format!("{} pending request count", self.kind),
            pending_total.clone(),
        );
        registry.register(
            format!("{}_requests_approved_total", self.kind),
            format!("{} approved request count", self.kind),
            approved_total.clone(),
        );
        registry.register(
            format!("{}_requests_declined_total", self.kind),
            format!("{} declined request count", self.kind),
            declined_total.clone(),
        );

        let agg_labels = OverseerrAggregateLabels {
            name: self.kind.clone(),
        };
        let mut pending = 0_f64;
        let mut approved = 0_f64;
        let mut declined = 0_f64;

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

            match request.status.as_f64() as i64 {
                1 => pending += 1.0,
                2 => approved += 1.0,
                3 => declined += 1.0,
                _ => {}
            }
        });

        requests_total
            .get_or_create(&agg_labels)
            .set(self.requests.len() as f64);
        pending_total.get_or_create(&agg_labels).set(pending);
        approved_total.get_or_create(&agg_labels).set(approved);
        declined_total.get_or_create(&agg_labels).set(declined);
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
        let session_location = Family::<SessionLocationLabels, Gauge<f64, AtomicU64>>::default();

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
        registry.register(
            format!("{prefix}_session_location"),
            format!("{prefix} session location"),
            session_location.clone(),
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

            let location_labels = SessionLocationLabels {
                name: self.name.clone(),
                user: session.user.clone(),
                title: session.title.clone(),
                city: session.location.city.clone(),
                country: session.location.country.clone(),
                latitude: session.location.latitude.clone(),
                longitude: session.location.longitude.clone(),
            };
            session_location.get_or_create(&location_labels).set(1.0);
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
    use crate::providers::overseerr::{MediaStatus, OverseerrRequest, RequestStatus};
    use crate::providers::radarr::RadarrMovie;
    use crate::providers::sonarr::SonarrEpisode;
    use crate::providers::structs::{Bandwidth, BandwidthLocation, Location, MediaType, Session,
        StreamDecision, User, LibraryCount};
    use crate::providers::tautulli::{SessionSummary, TautulliLocation};
    use crate::providers::structs::tautulli::Library as TautulliLibrary;
    use crate::tasks::{
        LibraryResult, OverseerrRequestResult, RadarrMovieResult, SessionResult, SonarrEpisodeResult,
        TautulliLibraryResult, TautulliSessionResult, TaskResult,
    };

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
        assert!(result.contains("homers_sonarr_today_episode{"));
        assert!(result.contains("sxe=\"S01E01\""));
        assert!(result.contains("serie=\"Test\""));
        assert!(result.contains("} 1.0\n"));
        assert!(result.contains("homers_sonarr_today_episodes_total{name=\"test\"} 1.0\n"));
    }

    fn make_tautulli_location(city: &str, country: &str, lat: &str, lon: &str) -> TautulliLocation {
        TautulliLocation {
            city: city.to_string(),
            country: country.to_string(),
            ip_address: "127.0.0.1".to_string(),
            latitude: lat.to_string(),
            longitude: lon.to_string(),
        }
    }

    fn make_location(city: &str, country: &str, lat: &str, lon: &str) -> Location {
        Location {
            city: city.to_string(),
            country: country.to_string(),
            ip_address: "127.0.0.1".to_string(),
            latitude: lat.to_string(),
            longitude: lon.to_string(),
        }
    }

    #[test]
    fn test_tautulli_session_format() {
        let result = format_metrics(vec![TaskResult::TautulliSession(TautulliSessionResult {
            sessions: vec![
                SessionSummary {
                    user: "alice".to_string(),
                    title: "Breaking Bad".to_string(),
                    state: "playing".to_string(),
                    progress: "42".to_string(),
                    quality: "1080p".to_string(),
                    quality_profile: "Original".to_string(),
                    video_stream: "direct play".to_string(),
                    media_type: "episode".to_string(),
                    season_number: Some("3".to_string()),
                    episode_number: Some("7".to_string()),
                    location: make_tautulli_location("Berlin", "Germany", "52.52", "13.40"),
                },
                SessionSummary {
                    user: "bob".to_string(),
                    title: "Inception".to_string(),
                    state: "paused".to_string(),
                    progress: "75".to_string(),
                    quality: "4K".to_string(),
                    quality_profile: "4K".to_string(),
                    video_stream: "transcode".to_string(),
                    media_type: "movie".to_string(),
                    season_number: None,
                    episode_number: None,
                    location: make_tautulli_location("Paris", "France", "48.85", "2.35"),
                },
            ],
        })])
        .unwrap();

        // session_count should be 2
        assert!(result.contains("homers_tautulli_session_count{} 2.0\n"));

        // session_info for alice
        assert!(result.contains("user=\"alice\""));
        assert!(result.contains("title=\"Breaking Bad\""));
        assert!(result.contains("state=\"playing\""));

        // session_progress for alice at 42%
        assert!(result.contains("homers_tautulli_session_progress{user=\"alice\",title=\"Breaking Bad\"} 42.0\n"));

        // session_progress for bob at 75%
        assert!(result.contains("homers_tautulli_session_progress{user=\"bob\",title=\"Inception\"} 75.0\n"));

        // session_location has geo labels
        assert!(result.contains("city=\"Berlin\""));
        assert!(result.contains("latitude=\"52.52\""));
        assert!(result.contains("longitude=\"13.40\""));
        assert!(result.contains("city=\"Paris\""));
    }

    #[test]
    fn test_tautulli_library_format() {
        let result = format_metrics(vec![TaskResult::TautulliLibrary(TautulliLibraryResult {
            libraries: vec![
                TautulliLibrary {
                    section_id: "1".to_string(),
                    section_name: "Movies".to_string(),
                    section_type: "movie".to_string(),
                    agent: "".to_string(),
                    thumb: "".to_string(),
                    art: "".to_string(),
                    count: "250".to_string(),
                    is_active: 1,
                    parent_count: None,
                    child_count: None,
                },
                TautulliLibrary {
                    section_id: "2".to_string(),
                    section_name: "TV Shows".to_string(),
                    section_type: "show".to_string(),
                    agent: "".to_string(),
                    thumb: "".to_string(),
                    art: "".to_string(),
                    count: "80".to_string(),
                    is_active: 1,
                    parent_count: Some("400".to_string()),
                    child_count: Some("3500".to_string()),
                },
                TautulliLibrary {
                    section_id: "3".to_string(),
                    section_name: "Archived".to_string(),
                    section_type: "movie".to_string(),
                    agent: "".to_string(),
                    thumb: "".to_string(),
                    art: "".to_string(),
                    count: "10".to_string(),
                    is_active: 0,
                    parent_count: None,
                    child_count: None,
                },
            ],
        })])
        .unwrap();

        // item_count metrics present
        assert!(result.contains("homers_tautulli_library_item_count{"));
        assert!(result.contains("section_name=\"Movies\""));
        assert!(result.contains("section_name=\"TV Shows\""));

        // child_count for TV Shows should be 3500
        assert!(result.contains("homers_tautulli_library_child_count{section_name=\"TV Shows\",section_type=\"show\"} 3500.0\n"));

        // parent_count for TV Shows should be 400
        assert!(result.contains("homers_tautulli_library_parent_count{section_name=\"TV Shows\",section_type=\"show\"} 400.0\n"));

        // item_count for Movies should be 250
        assert!(result.contains("homers_tautulli_library_item_count{section_name=\"Movies\",section_type=\"movie\"} 250.0\n"));

        // active library shows 1.0, inactive shows 0.0
        assert!(result.contains("homers_tautulli_library_active{section_name=\"Movies\",section_type=\"movie\"} 1.0\n"));
        assert!(result.contains("homers_tautulli_library_active{section_name=\"Archived\",section_type=\"movie\"} 0.0\n"));
    }

    #[test]
    fn test_tautulli_history_format() {
        use crate::providers::tautulli::HistoryEntry;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let result = format_metrics(vec![TaskResult::TautulliHistory(TautulliHistoryResult {
            total_plays: 5000,
            entries: vec![
                HistoryEntry {
                    date: now - 3600,
                    started: now - 3600,
                    stopped: now - 1800,
                    duration: 1800,
                    user: "alice".to_string(),
                    friendly_name: "Alice".to_string(),
                    full_title: "Breaking Bad - S01E01 - Pilot".to_string(),
                    title: "Pilot".to_string(),
                    parent_title: "Season 1".to_string(),
                    grandparent_title: "Breaking Bad".to_string(),
                    media_type: "episode".to_string(),
                    platform: "Chrome".to_string(),
                    player: "Plex Web".to_string(),
                    year: 2008,
                    percent_complete: 100,
                    watched_status: 1.0,
                    transcode_decision: "direct play".to_string(),
                },
                HistoryEntry {
                    date: now - 7200,
                    started: now - 7200,
                    stopped: now - 5400,
                    duration: 1800,
                    user: "alice".to_string(),
                    friendly_name: "Alice".to_string(),
                    full_title: "Inception".to_string(),
                    title: "Inception".to_string(),
                    parent_title: "".to_string(),
                    grandparent_title: "".to_string(),
                    media_type: "movie".to_string(),
                    platform: "Firefox".to_string(),
                    player: "Plex Web".to_string(),
                    year: 2010,
                    percent_complete: 50,
                    watched_status: 0.5,
                    transcode_decision: "transcode".to_string(),
                },
                HistoryEntry {
                    date: now - 100000, // older than 24h
                    started: now - 100000,
                    stopped: now - 98200,
                    duration: 1800,
                    user: "bob".to_string(),
                    friendly_name: "Bob".to_string(),
                    full_title: "Old Movie".to_string(),
                    title: "Old Movie".to_string(),
                    parent_title: "".to_string(),
                    grandparent_title: "".to_string(),
                    media_type: "movie".to_string(),
                    platform: "TV".to_string(),
                    player: "Plex".to_string(),
                    year: 2000,
                    percent_complete: 100,
                    watched_status: 1.0,
                    transcode_decision: "direct play".to_string(),
                },
            ],
        })])
        .unwrap();

        // total_plays should be 5000
        assert!(result.contains("homers_tautulli_history_total_plays 5000.0\n"));

        // plays_24h should be 2 (only first two entries are within 24h)
        assert!(result.contains("homers_tautulli_history_plays_24h 2.0\n"));

        // user_watches_24h for Alice should be 2
        assert!(result.contains(
            "homers_tautulli_history_user_watches_24h{user=\"Alice\"} 2.0\n"
        ));

        // Bob's entry is older than 24h so shouldn't appear in user_watches_24h
        assert!(!result.contains("user=\"Bob\""));
    }

    #[test]
    fn test_radarr_format() {
        let result = format_metrics(vec![TaskResult::Radarr(RadarrMovieResult {
            name: "radarr-main".to_string(),
            movies: vec![
                RadarrMovie {
                    title: "The Matrix".to_string(),
                    has_file: true,
                    monitored: true,
                    is_available: true,
                    missing_available: false,
                },
                RadarrMovie {
                    title: "Dune".to_string(),
                    has_file: false,
                    monitored: true,
                    is_available: true,
                    missing_available: true,
                },
                RadarrMovie {
                    title: "Old Film".to_string(),
                    has_file: false,
                    monitored: false,
                    is_available: false,
                    missing_available: false,
                },
            ],
        })])
        .unwrap();

        // has_file: The Matrix = 1.0, Dune = 0.0
        assert!(result.contains("homers_radarr_movie_has_file{name=\"radarr-main\",title=\"The Matrix\"} 1.0\n"));
        assert!(result.contains("homers_radarr_movie_has_file{name=\"radarr-main\",title=\"Dune\"} 0.0\n"));

        // monitored: The Matrix = 1.0, Dune = 1.0, Old Film = 0.0
        assert!(result.contains("homers_radarr_movie_monitored{name=\"radarr-main\",title=\"The Matrix\"} 1.0\n"));
        assert!(result.contains("homers_radarr_movie_monitored{name=\"radarr-main\",title=\"Dune\"} 1.0\n"));
        assert!(result.contains("homers_radarr_movie_monitored{name=\"radarr-main\",title=\"Old Film\"} 0.0\n"));

        // available: The Matrix = 1.0, Dune = 1.0, Old Film = 0.0
        assert!(result.contains("homers_radarr_movie_available{name=\"radarr-main\",title=\"The Matrix\"} 1.0\n"));
        assert!(result.contains("homers_radarr_movie_available{name=\"radarr-main\",title=\"Old Film\"} 0.0\n"));

        // aggregate counts: total=3, monitored=2, missing=1
        assert!(result.contains("homers_radarr_movies_total{name=\"radarr-main\"} 3.0\n"));
        assert!(result.contains("homers_radarr_movies_monitored_total{name=\"radarr-main\"} 2.0\n"));
        assert!(result.contains("homers_radarr_movies_missing_total{name=\"radarr-main\"} 1.0\n"));
    }

    #[test]
    fn test_overseerr_format() {
        let result = format_metrics(vec![TaskResult::Overseerr(OverseerrRequestResult {
            kind: "overseerr".to_string(),
            requests: vec![
                OverseerrRequest {
                    media_type: "movie".to_string(),
                    media_id: 1,
                    status: RequestStatus::Pending,
                    requested_by: "alice".to_string(),
                    media_status: MediaStatus::Processing,
                    media_title: "Dune Part Two".to_string(),
                    requested_at: "2024-01-01T00:00:00Z".to_string(),
                },
                OverseerrRequest {
                    media_type: "tv".to_string(),
                    media_id: 2,
                    status: RequestStatus::Approved,
                    requested_by: "bob".to_string(),
                    media_status: MediaStatus::Available,
                    media_title: "Shogun".to_string(),
                    requested_at: "2024-01-02T00:00:00Z".to_string(),
                },
                OverseerrRequest {
                    media_type: "movie".to_string(),
                    media_id: 3,
                    status: RequestStatus::Declined,
                    requested_by: "carol".to_string(),
                    media_status: MediaStatus::Unknown,
                    media_title: "Bad Movie".to_string(),
                    requested_at: "2024-01-03T00:00:00Z".to_string(),
                },
            ],
        })])
        .unwrap();

        // request_status metric present with correct labels
        assert!(result.contains("homers_overseerr_request_status{"));
        assert!(result.contains("media_title=\"Dune Part Two\""));
        assert!(result.contains("requested_by=\"alice\""));

        // Pending=1.0, Approved=2.0, Declined=3.0
        assert!(result.contains("homers_overseerr_request_status{media_type=\"movie\",requested_by=\"alice\",media_title=\"Dune Part Two\"} 1.0\n"));
        assert!(result.contains("homers_overseerr_request_status{media_type=\"tv\",requested_by=\"bob\",media_title=\"Shogun\"} 2.0\n"));
        assert!(result.contains("homers_overseerr_request_status{media_type=\"movie\",requested_by=\"carol\",media_title=\"Bad Movie\"} 3.0\n"));

        // media_status: Processing=3.0, Available=5.0, Unknown=1.0
        assert!(result.contains("homers_overseerr_media_status{media_type=\"movie\",requested_by=\"alice\",media_title=\"Dune Part Two\"} 3.0\n"));
        assert!(result.contains("homers_overseerr_media_status{media_type=\"tv\",requested_by=\"bob\",media_title=\"Shogun\"} 5.0\n"));
        assert!(result.contains("homers_overseerr_media_status{media_type=\"movie\",requested_by=\"carol\",media_title=\"Bad Movie\"} 1.0\n"));

        // aggregate counts
        assert!(result.contains("homers_overseerr_requests_total{name=\"overseerr\"} 3.0\n"));
        assert!(result.contains("homers_overseerr_requests_pending_total{name=\"overseerr\"} 1.0\n"));
        assert!(result.contains("homers_overseerr_requests_approved_total{name=\"overseerr\"} 1.0\n"));
        assert!(result.contains("homers_overseerr_requests_declined_total{name=\"overseerr\"} 1.0\n"));
    }

    #[test]
    fn test_plex_session_format() {
        let result = format_metrics(vec![TaskResult::PlexSession(SessionResult {
            name: "plex-home".to_string(),
            kind: "plex".to_string(),
            users: vec![
                User { name: "alice".to_string() },
                User { name: "bob".to_string() },
                User { name: "inactive-user".to_string() },
            ],
            sessions: vec![
                Session {
                    title: "The Wire".to_string(),
                    user: "alice".to_string(),
                    stream_decision: StreamDecision::DirectPlay,
                    media_type: "episode".to_string(),
                    state: "playing".to_string(),
                    progress: 55.0,
                    quality: "1080p".to_string(),
                    season_number: Some("1".to_string()),
                    episode_number: Some("3".to_string()),
                    address: "192.168.1.10".to_string(),
                    location: make_location("New York", "US", "40.71", "-74.00"),
                    local: true,
                    secure: true,
                    relayed: false,
                    platform: "Chrome".to_string(),
                    bandwidth: Bandwidth {
                        bandwidth: 8000,
                        location: BandwidthLocation::Lan,
                    },
                },
                Session {
                    title: "Interstellar".to_string(),
                    user: "bob".to_string(),
                    stream_decision: StreamDecision::Transcode,
                    media_type: "movie".to_string(),
                    state: "playing".to_string(),
                    progress: 30.0,
                    quality: "4K".to_string(),
                    season_number: None,
                    episode_number: None,
                    address: "203.0.113.5".to_string(),
                    location: make_location("London", "UK", "51.50", "-0.12"),
                    local: false,
                    secure: true,
                    relayed: false,
                    platform: "Plex for Android".to_string(),
                    bandwidth: Bandwidth {
                        bandwidth: 20000,
                        location: BandwidthLocation::Wan,
                    },
                },
            ],
        })])
        .unwrap();

        // session_count should be 2
        assert!(result.contains("homers_plex_session_count{name=\"plex-home\"} 2.0\n"));

        // session_info present
        assert!(result.contains("homers_plex_session_info{"));
        assert!(result.contains("title=\"The Wire\""));
        assert!(result.contains("title=\"Interstellar\""));

        // session_progress values
        assert!(result.contains("homers_plex_session_progress{name=\"plex-home\",user=\"alice\",title=\"The Wire\"} 55.0\n"));
        assert!(result.contains("homers_plex_session_progress{name=\"plex-home\",user=\"bob\",title=\"Interstellar\"} 30.0\n"));

        // user_active: alice and bob are active (1.0), inactive-user is 0.0
        assert!(result.contains("homers_plex_user_active{name=\"plex-home\",user=\"alice\"} 1.0\n"));
        assert!(result.contains("homers_plex_user_active{name=\"plex-home\",user=\"bob\"} 1.0\n"));
        assert!(result.contains("homers_plex_user_active{name=\"plex-home\",user=\"inactive-user\"} 0.0\n"));

        // bandwidth: LAN=8000, WAN=20000
        assert!(result.contains("homers_plex_session_bandwidth{name=\"plex-home\",location=\"LAN\"} 8000.0\n"));
        assert!(result.contains("homers_plex_session_bandwidth{name=\"plex-home\",location=\"WAN\"} 20000.0\n"));

        // session_location geo labels
        assert!(result.contains("city=\"New York\""));
        assert!(result.contains("latitude=\"40.71\""));
        assert!(result.contains("longitude=\"-74.00\""));
    }

    #[test]
    fn test_plex_library_format() {
        let result = format_metrics(vec![TaskResult::PlexLibrary(LibraryResult {
            name: "plex-home".to_string(),
            kind: "plex".to_string(),
            libraries: vec![
                LibraryCount {
                    name: "Movies".to_string(),
                    media_type: MediaType::Movie,
                    count: 500,
                    child_count: None,
                    grand_child_count: None,
                },
                LibraryCount {
                    name: "4K Movies".to_string(),
                    media_type: MediaType::Movie,
                    count: 150,
                    child_count: None,
                    grand_child_count: None,
                },
                LibraryCount {
                    name: "TV Shows".to_string(),
                    media_type: MediaType::Show,
                    count: 120,
                    child_count: Some(600),
                    grand_child_count: Some(8000),
                },
            ],
        })])
        .unwrap();

        // Aggregate movie count: 500 + 150 = 650
        assert!(result.contains("homers_plex_movie_count{} 650.0\n"));

        // Aggregate show/season/episode counts
        assert!(result.contains("homers_plex_show_count{} 120.0\n"));
        assert!(result.contains("homers_plex_season_count{} 600.0\n"));
        assert!(result.contains("homers_plex_episode_count{} 8000.0\n"));

        // Per-library counts
        assert!(result.contains("homers_plex_library_count{name=\"plex-home\",library_name=\"Movies\",library_type=\"Movie\"} 500.0\n"));
        assert!(result.contains("homers_plex_library_count{name=\"plex-home\",library_name=\"4K Movies\",library_type=\"Movie\"} 150.0\n"));
        assert!(result.contains("homers_plex_library_count{name=\"plex-home\",library_name=\"TV Shows\",library_type=\"Show\"} 120.0\n"));
    }

    #[test]
    fn test_empty_results() {
        // Empty Tautulli sessions
        let result = format_metrics(vec![TaskResult::TautulliSession(TautulliSessionResult {
            sessions: vec![],
        })])
        .unwrap();
        assert!(result.contains("homers_tautulli_session_count{} 0.0\n"));

        // Empty Radarr movies
        let result = format_metrics(vec![TaskResult::Radarr(RadarrMovieResult {
            name: "radarr".to_string(),
            movies: vec![],
        })])
        .unwrap();
        assert!(result.contains("homers_radarr_movies_total{name=\"radarr\"} 0.0\n"));
        assert!(result.contains("homers_radarr_movies_monitored_total{name=\"radarr\"} 0.0\n"));
        assert!(result.contains("homers_radarr_movies_missing_total{name=\"radarr\"} 0.0\n"));
        // No per-movie metrics should be present
        assert!(!result.contains("homers_radarr_movie_has_file{"));

        // Empty Overseerr requests
        let result = format_metrics(vec![TaskResult::Overseerr(OverseerrRequestResult {
            kind: "overseerr".to_string(),
            requests: vec![],
        })])
        .unwrap();
        assert!(result.contains("homers_overseerr_requests_total{name=\"overseerr\"} 0.0\n"));
        assert!(result.contains("homers_overseerr_requests_pending_total{name=\"overseerr\"} 0.0\n"));
        assert!(result.contains("homers_overseerr_requests_approved_total{name=\"overseerr\"} 0.0\n"));
        assert!(result.contains("homers_overseerr_requests_declined_total{name=\"overseerr\"} 0.0\n"));

        // Empty Plex sessions — session_count still emitted via the loop-independent set
        let result = format_metrics(vec![TaskResult::PlexSession(SessionResult {
            name: "plex".to_string(),
            kind: "plex".to_string(),
            users: vec![],
            sessions: vec![],
        })])
        .unwrap();
        assert!(result.contains("homers_plex_session_count{name=\"plex\"} 0.0\n"));

        // Empty Tautulli libraries
        let result = format_metrics(vec![TaskResult::TautulliLibrary(TautulliLibraryResult {
            libraries: vec![],
        })])
        .unwrap();
        // Registry is created but no time-series emitted for per-library metrics
        assert!(!result.contains("homers_tautulli_library_item_count{section_name="));
    }

    #[test]
    fn test_special_characters_in_title() {
        // prometheus-client handles label escaping natively; verify output is parseable
        let result = format_metrics(vec![TaskResult::Radarr(RadarrMovieResult {
            name: "radarr".to_string(),
            movies: vec![
                RadarrMovie {
                    title: "Movie with \"quotes\"".to_string(),
                    has_file: true,
                    monitored: true,
                    is_available: true,
                    missing_available: false,
                },
                RadarrMovie {
                    title: "Movie with \\backslash".to_string(),
                    has_file: false,
                    monitored: false,
                    is_available: false,
                    missing_available: false,
                },
            ],
        })])
        .unwrap();

        // The output must contain both entries (escaping handled by prometheus-client)
        assert!(result.contains("homers_radarr_movie_has_file{"));
        assert!(result.contains("homers_radarr_movies_total{name=\"radarr\"} 2.0\n"));

        // prometheus-client encodes quotes as literal " inside label values and
        // backslashes as single \ — verify both titles appear in the output
        assert!(result.contains("title=\"Movie with \"quotes\"\""));
        assert!(result.contains("title=\"Movie with \\backslash\""));
    }
}
