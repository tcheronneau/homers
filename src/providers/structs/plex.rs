use ipgeolocate::{Locator, Service};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    SessionMetadata(SessionMetadata),
    HistoryMetadata(HistoryMetadata),
    LibraryMetadata(LibraryMetadata),
    Default(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MediaContainer {
    LibraryContainer(LibraryContainer),
    LibraryItemsContainer(LibraryItemsContainer),
    ActivityContainer(ActivityContainer),
    Default(serde_json::Value),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryType {
    Show,
    Movie,
    Photo,
    Artist,
    #[default]
    Default,
}
impl Display for LibraryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryType::Show => write!(f, "Show"),
            LibraryType::Movie => write!(f, "Movie"),
            LibraryType::Photo => write!(f, "Photo"),
            LibraryType::Artist => write!(f, "Music"),
            LibraryType::Default => write!(f, "Unknown"),
        }
    }
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
pub struct ActivityContainer {
    pub size: i64,
    #[serde(rename = "Metadata")]
    #[serde(default)]
    pub metadata: Vec<Metadata>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryContainer {
    pub size: i64,
    #[serde(rename = "Directory")]
    pub directory: Vec<Directory>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemsContainer {
    pub size: i64,
    pub allow_sync: bool,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: i64,
    pub library_section_title: String,
    #[serde(rename = "librarySectionUUID")]
    pub library_section_uuid: String,
    #[serde(rename = "Metadata")]
    pub metadata: Vec<Metadata>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    pub key: String,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryMetadata {
    #[serde(rename = "type")]
    pub type_field: String,
    pub title: String,
    pub leaf_count: Option<i64>,
    pub child_count: Option<i64>,
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
    pub original_title: Option<String>,
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
        let user = self.user.title.clone();
        let state = self.player.state_field.clone();
        let progress = self.progress();
        let part = &self.media[0].part[0];
        let video_stream: &Stream = &part.stream.iter().find(|s| s.stream_type == 1).unwrap();
        let quality = video_stream.display_title.to_string();
        let season_number = match self.parent_index {
            Some(index) => Some(index.to_string()),
            None => None,
        };
        let episode_number = match self.index {
            Some(index) => Some(index.to_string()),
            None => None,
        };
        let location = get_ip_info(&self.player.remote_public_address).await;
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
        let address = self.player.address.clone();
        let local = self.player.local;
        let secure = self.player.secure;
        let relayed = self.player.relayed;
        let platform = self.player.platform.clone();
        let title = match &self.grand_parent_title {
            Some(parent) => parent.to_string(),
            None => self.title.clone(),
        };
        let bandwidth = Bandwidth {
            bandwidth: self.session.bandwidth,
            location: self.session.location.clone().into(),
        };
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
            bandwidth,
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
    pub stream_type: i64,
    pub decision: Option<String>,
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
    pub bandwidth: i64,
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
    pub bandwidth: Bandwidth,
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
