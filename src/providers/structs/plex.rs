use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlexResponse {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub history_key: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub rating_key: Option<String>,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: String,
    #[serde(default)]
    pub parent_key: Option<String>,
    #[serde(default)]
    pub grandparent_key: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub grandparent_title: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub thumb: Option<String>,
    #[serde(default)]
    pub parent_thumb: Option<String>,
    #[serde(default)]
    pub grandparent_thumb: Option<String>,
    #[serde(default)]
    pub grandparent_art: Option<String>,
    #[serde(default)]
    pub index: Option<i64>,
    #[serde(default)]
    pub parent_index: Option<i64>,
    #[serde(default)]
    pub originally_available_at: Option<String>,
    pub viewed_at: i64,
    #[serde(rename = "accountID")]
    pub account_id: i64,
    #[serde(default, rename = "deviceID")]
    pub device_id: Option<i64>,
}
impl std::fmt::Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.title {
            None => write!(f, "Unknown viewed at {}", self.viewed_at),
            Some(title) => write!(f, "{} viewed at {}", title, self.viewed_at),
        }
    }
}
