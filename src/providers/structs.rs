use serde::{Deserialize, Serialize};
use std::fmt::Display;
pub mod jellyfin;
pub mod overseerr;
pub mod plex;
pub mod radarr;
pub mod sonarr;
pub mod tautulli;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub title: String,
    pub user: String,
    pub stream_decision: StreamDecision,
    pub media_type: String,
    pub state: String,
    pub progress: i64,
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
impl From<jellyfin::SessionResponse> for Session {
    fn from(session: jellyfin::SessionResponse) -> Self {
        let mut title = "".to_string();
        let mut media_type = "Unknown".to_string();
        match &session.now_playing_item {
            Some(item) => {
                title = item.name.clone();
                media_type = item.type_field.clone();
            }
            None => (),
        };
        let progress = match &session.play_state.posititon_ticks {
            Some(progress) => (progress / &session.now_playing_item.unwrap().run_time_ticks) * 100,
            None => 0,
        };
        let state = match &session.play_state.is_paused {
            Some(paused) => match paused {
                true => "Paused",
                false => "Playing",
            },
            None => "",
        };
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
            None => StreamDecision::Transcode,
        };

        Session {
            title: title.to_string(),
            user: session.user_name,
            stream_decision,
            media_type,
            state: state.to_string(),
            progress,
            quality: "".to_string(),
            season_number: None,
            episode_number: None,
            address: session.remote_end_point,
            location: Location {
                city: "".to_string(),
                country: "".to_string(),
                ip_address: "".to_string(),
                latitude: "".to_string(),
                longitude: "".to_string(),
            },
            local: false,
            secure: false,
            relayed: false,
            platform: session.client,
            bandwidth: Bandwidth {
                bandwidth: 0,
                location: BandwidthLocation::Wan,
            },
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
}
impl From<String> for BandwidthLocation {
    fn from(location: String) -> Self {
        match location.as_str() {
            "wan" => BandwidthLocation::Wan,
            "lan" => BandwidthLocation::Lan,
            _ => BandwidthLocation::Wan,
        }
    }
}
impl Display for BandwidthLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BandwidthLocation::Wan => write!(f, "WAN"),
            BandwidthLocation::Lan => write!(f, "LAN"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamDecision {
    DirectPlay,
    DirectStream,
    Transcode,
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
