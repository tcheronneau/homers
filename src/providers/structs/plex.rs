use serde::{Deserialize, Serialize};

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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    #[serde(rename = "Part")]
    pub part: Vec<Part>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub decision: String,
    pub container: String,
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
