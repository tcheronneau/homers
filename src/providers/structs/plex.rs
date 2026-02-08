use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlexResponse {
    //#[serde(rename = "MediaContainer",deserialize_with = "deserialize_media_container")]
    #[serde(rename = "MediaContainer")]
    pub media_container: MediaContainer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MediaContainer {
    // Order matters with untagged enums - most specific variants first
    // LibraryItemsContainer has unique required fields (library_section_id, library_section_title, library_section_uuid)
    LibraryItemsContainer(LibraryItemsContainer),
    // StatisticsContainer has Account field
    StatisticsContainer(StatisticsContainer),
    // LibraryContainer has Directory field
    LibraryContainer(LibraryContainer),
    // ActivityContainer has Metadata field (with default, so relatively permissive)
    ActivityContainer(ActivityContainer),
    // Default catches everything else (most permissive, must be last)
    Default(MediaContainerDefault),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    // Order matters with untagged enums - most specific variants first
    // SessionMetadata has many unique required fields (Media, User, Player, Session)
    SessionMetadata(SessionMetadata),
    // HistoryMetadata has unique history_key field
    HistoryMetadata(HistoryMetadata),
    // LibraryMetadata is simpler but has title field
    LibraryMetadata(LibraryMetadata),
    // Default catches everything else
    Default(serde_json::Value),
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
pub struct MediaContainerDefault {
    pub size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsContainer {
    pub size: i64,
    #[serde(rename = "Account")]
    pub account: Vec<StatUser>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryContainer {
    pub size: i64,
    #[serde(rename = "Directory")]
    pub directory: Vec<Directory>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemsContainer {
    pub size: i64,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: i64,
    pub library_section_title: String,
    #[serde(rename = "librarySectionUUID")]
    pub library_section_uuid: String,
    #[serde(rename = "Metadata")]
    #[serde(default)]
    pub metadata: Vec<Metadata>,
}
impl Default for LibraryItemsContainer {
    fn default() -> Self {
        LibraryItemsContainer {
            size: 0,
            library_section_id: 0,
            library_section_title: "".to_string(),
            library_section_uuid: "".to_string(),
            metadata: vec![],
        }
    }
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
pub struct StatUser {
    pub name: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub title: String,
}
impl From<StatUser> for User {
    fn from(stat_user: StatUser) -> Self {
        User {
            title: stat_user.name,
        }
    }
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
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LibraryInfos {
    pub library_name: String,
    pub library_type: String,
    pub library_size: i64,
    pub library_child_size: Option<i64>,
    pub library_grand_child_size: Option<i64>,
}
