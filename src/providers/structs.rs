use async_trait::async_trait;
use ipgeolocate::{Locator, Service};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
pub mod jellyfin;
pub mod overseerr;
pub mod plex;
pub mod radarr;
pub mod sonarr;
pub mod tautulli;

#[async_trait]
pub trait AsyncFrom<T>: Sized {
    async fn from_async(value: T) -> Self;
}

async fn get_ip_info(ip: &str) -> Location {
    let service = Service::IpApi;
    match Locator::get(ip, service).await {
        Ok(location) => Location {
            city: location.city,
            country: location.country,
            ip_address: ip.to_string(),
            latitude: location.latitude,
            longitude: location.longitude,
        },
        Err(_) => Location {
            city: "Unknown".to_string(),
            country: "Unknown".to_string(),
            ip_address: ip.to_string(),
            latitude: "0.0".to_string(),
            longitude: "0.0".to_string(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub name: String,
}
impl From<plex::User> for User {
    fn from(user: plex::User) -> Self {
        User { name: user.title }
    }
}
impl From<jellyfin::User> for User {
    fn from(user: jellyfin::User) -> Self {
        User { name: user.name }
    }
}
impl From<plex::StatUser> for User {
    fn from(stat_user: plex::StatUser) -> Self {
        User {
            name: stat_user.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub title: String,
    pub user: String,
    pub stream_decision: StreamDecision,
    pub media_type: String,
    pub state: String,
    pub progress: f64,
    pub quality: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub address: String,
    pub location: Location,
    pub local: bool,
    pub secure: bool,
    pub relayed: bool,
    pub platform: String,
    pub bandwidth: Bandwidth,
}
#[async_trait]
impl AsyncFrom<jellyfin::SessionResponse> for Session {
    async fn from_async(session: jellyfin::SessionResponse) -> Self {
        let mut title = "".to_string();
        let mut media_type = "Unknown".to_string();
        let mut quality = "".to_string();
        let mut episode_number = None;
        let mut season_number = None;
        match &session.now_playing_item {
            Some(item) => {
                title = item.name.clone();
                media_type = item.type_field.clone();
                episode_number = match item.index_number {
                    Some(index) => Some(index.to_string()),
                    None => None,
                };
                season_number = match item.parent_index_number {
                    Some(index) => Some(index.to_string()),
                    None => None,
                };
                let media_stream = &item
                    .media_streams
                    .iter()
                    .find(|stream| stream.type_field == "Video");
                quality = match media_stream {
                    Some(stream) => match &stream.display_title {
                        Some(title) => title.to_string(),
                        None => match &stream.title {
                            Some(title) => title.to_string(),
                            None => "Unknown".to_string(),
                        },
                    },
                    None => "Unknown".to_string(),
                }
            }
            None => (),
        };
        let progress = match &session.play_state.position_ticks {
            Some(position) => match &session.now_playing_item {
                Some(item) => (*position as f64 / item.run_time_ticks as f64) * 100.0,
                None => 0.0,
            },
            None => 0.0,
        };
        let state = match &session.play_state.is_paused {
            Some(paused) => match paused {
                true => "Paused",
                false => {
                    if session.now_playing_item.is_some() {
                        "Playing"
                    } else {
                        "Idle"
                    }
                }
            },
            None => "Idle",
        };
        let location = get_ip_info(&session.remote_end_point).await;
        let stream_decision = match &session.play_state.play_method {
            Some(method) => match method.as_str() {
                "DirectPlay" => StreamDecision::DirectPlay,
                "Transcode" => match &session.transcoding_info {
                    Some(transcoding_info) => {
                        if transcoding_info.is_video_direct {
                            StreamDecision::DirectStream
                        } else {
                            StreamDecision::Transcode
                        }
                    }
                    None => StreamDecision::Transcode,
                },
                _ => StreamDecision::Transcode,
            },
            None => StreamDecision::None,
        };

        Session {
            title: title.to_string(),
            user: session.user_name,
            stream_decision,
            media_type,
            state: state.to_string(),
            progress,
            quality,
            season_number,
            episode_number,
            address: session.remote_end_point,
            location,
            local: false,
            secure: false,
            relayed: false,
            platform: session.client,
            bandwidth: Bandwidth {
                bandwidth: -1,
                location: BandwidthLocation::Unknown,
            },
        }
    }
}
#[async_trait]
impl AsyncFrom<plex::SessionMetadata> for Session {
    async fn from_async(session: plex::SessionMetadata) -> Self {
        let media_type = session.type_field.clone();
        let user = session.user.title.clone();
        let state = session.player.state_field.clone();
        let progress = session.progress();
        let part = &session.media[0].part[0];
        let video_stream: &plex::Stream = &part.stream.iter().find(|s| s.stream_type == 1).unwrap();
        let quality = video_stream.display_title.to_string();
        let season_number = match session.parent_index {
            Some(index) => Some(index.to_string()),
            None => None,
        };
        let episode_number = match session.index {
            Some(index) => Some(index.to_string()),
            None => None,
        };
        let location = get_ip_info(&session.player.remote_public_address).await;
        let decision = part.decision.clone();
        let video_stream_decision = match &video_stream.decision {
            Some(decision) => decision.to_string(),
            None => "transcode".to_string(),
        };
        let stream_decision = match decision.as_str() {
            "directplay" => StreamDecision::DirectPlay,
            "transcode" => match video_stream_decision.as_str() {
                "copy" => StreamDecision::DirectStream,
                _ => StreamDecision::Transcode,
            },
            _ => StreamDecision::Transcode,
        };
        let address = session.player.address.clone();
        let local = session.player.local;
        let secure = session.player.secure;
        let relayed = session.player.relayed;
        let platform = session.player.platform.clone();
        let title = match &session.grand_parent_title {
            Some(parent) => parent.to_string(),
            None => session.title.clone(),
        };
        let bandwidth = Bandwidth {
            bandwidth: session.session.bandwidth,
            location: session.session.location.clone().into(),
        };
        Session {
            title,
            user,
            stream_decision,
            media_type,
            state,
            progress: progress as f64,
            quality,
            season_number,
            episode_number,
            location,
            address,
            local,
            secure,
            relayed,
            platform,
            bandwidth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bandwidth {
    pub bandwidth: i64,
    pub location: BandwidthLocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BandwidthLocation {
    Wan,
    Lan,
    Unknown,
}
impl From<String> for BandwidthLocation {
    fn from(location: String) -> Self {
        match location.as_str() {
            "wan" => BandwidthLocation::Wan,
            "lan" => BandwidthLocation::Lan,
            _ => BandwidthLocation::Unknown,
        }
    }
}
impl Display for BandwidthLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BandwidthLocation::Wan => write!(f, "WAN"),
            BandwidthLocation::Lan => write!(f, "LAN"),
            BandwidthLocation::Unknown => write!(f, "Undefined"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamDecision {
    DirectPlay,
    DirectStream,
    Transcode,
    None,
}
impl From<String> for StreamDecision {
    fn from(decision: String) -> Self {
        match decision.as_str() {
            "directplay" => StreamDecision::DirectPlay,
            "directstream" => StreamDecision::DirectStream,
            "transcode" => StreamDecision::Transcode,
            _ => StreamDecision::Transcode,
        }
    }
}
impl Display for StreamDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamDecision::DirectPlay => write!(f, "Direct Play"),
            StreamDecision::DirectStream => write!(f, "Direct Stream"),
            StreamDecision::Transcode => write!(f, "Transcode"),
            StreamDecision::None => write!(f, "None"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub city: String,
    pub country: String,
    pub ip_address: String,
    pub latitude: String,
    pub longitude: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Show,
    Music,
    Book,
    Unknown,
}
impl From<String> for MediaType {
    fn from(media_type: String) -> Self {
        match media_type.as_str() {
            "movie" => MediaType::Movie,
            "show" | "shows" => MediaType::Show,
            "music" => MediaType::Music,
            "book" => MediaType::Book,
            _ => MediaType::Unknown,
        }
    }
}
impl ToString for MediaType {
    fn to_string(&self) -> String {
        match self {
            MediaType::Movie => "Movie".to_string(),
            MediaType::Show => "Show".to_string(),
            MediaType::Music => "Music".to_string(),
            MediaType::Book => "Book".to_string(),
            MediaType::Unknown => "Unknown".to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryCount {
    pub name: String,
    pub media_type: MediaType,
    pub count: i64,
    pub child_count: Option<i64>,
    pub grand_child_count: Option<i64>,
}
impl From<plex::LibraryInfos> for LibraryCount {
    fn from(library: plex::LibraryInfos) -> Self {
        LibraryCount {
            name: library.library_name,
            media_type: library.library_type.into(),
            count: library.library_size,
            child_count: library.library_child_size,
            grand_child_count: library.library_grand_child_size,
        }
    }
}
impl From<jellyfin::LibraryInfos> for LibraryCount {
    fn from(counts: jellyfin::LibraryInfos) -> Self {
        LibraryCount {
            name: counts.name,
            media_type: counts.library_type.to_lowercase().into(),
            count: counts.count,
            child_count: counts.child_count,
            grand_child_count: counts.grand_child_count,
        }
    }
}
