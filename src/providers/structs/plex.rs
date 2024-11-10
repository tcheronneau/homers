use ipgeolocate::{Locator, Service};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    SessionMetadata(SessionMetadata),
    HistoryMetadata(HistoryMetadata),
    Default(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlexResponse {
    //#[serde(rename = "MediaContainer",deserialize_with = "deserialize_media_container")]
    #[serde(rename = "MediaContainer")]
    pub media_container: MediaContainer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer {
    pub size: i64,
    #[serde(rename = "Metadata")]
    pub metadata: Vec<Metadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryMetadata {
    #[serde(rename = "type")]
    pub type_field: String,
    pub history_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetadata {
    pub title: String,
    pub parent_title: Option<String>,
    pub grand_parent_title: Option<String>,
    pub index: Option<i64>,
    pub parent_index: Option<i64>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "Media")]
    pub media: Vec<Media>,
    #[serde(rename = "User")]
    pub user: User,
    #[serde(rename = "Player")]
    pub player: Player,
    #[serde(rename = "Session")]
    pub session: Session,
    pub view_offset: i64,
}
impl SessionMetadata {
    pub fn progress(&self) -> i64 {
        let duration = self.media[0].duration;
        let offset = self.view_offset;
        let progress = (offset as f64 / duration as f64) * 100.0;
        progress as i64
    }
    pub async fn to(&self) -> PlexSessions {
        let media_type = self.type_field.clone();
        let title = match &self.grand_parent_title {
            Some(title) => title.clone(),
            None => match &self.parent_title {
                Some(title) => title.clone(),
                None => self.title.clone(),
            },
        };
        let user = self.user.title.clone();
        let state = self.player.state_field.clone();
        let progress = self.progress();
        let quality = self.media[0].part[0].stream[0].display_title.clone();
        let season_number = match self.index {
            Some(index) => Some(index.to_string()),
            None => None,
        };
        let episode_number = match self.parent_index {
            Some(index) => Some(index.to_string()),
            None => None,
        };
        let location = get_ip_info(&self.player.remote_public_address).await;
        let stream_decision = self.media[0].part[0].decision.clone().into();
        let address = self.player.address.clone();
        let local = self.player.local;
        let secure = self.player.secure;
        let relayed = self.player.relayed;
        let platform = self.player.platform.clone();
        PlexSessions {
            title,
            user,
            stream_decision,
            media_type,
            state,
            progress,
            quality,
            season_number,
            episode_number,
            location,
            address,
            local,
            secure,
            relayed,
            platform,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    #[serde(rename = "Part")]
    pub part: Vec<Part>,
    pub duration: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub decision: String,
    pub container: String,
    #[serde(rename = "Stream")]
    pub stream: Vec<Stream>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub display_title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub platform: String,
    #[serde(rename = "state")]
    pub state_field: String,
    pub local: bool,
    pub remote_public_address: String,
    pub relayed: bool,
    pub secure: bool,
    pub product: String,
    pub address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub location: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub city: String,
    pub country: String,
    pub ip_address: String,
    pub latitude: String,
    pub longitude: String,
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
pub struct PlexSessions {
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
