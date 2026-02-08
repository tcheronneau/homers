use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub artist_name: String,
    #[serde(default)]
    pub monitored: bool,
    #[serde(default)]
    pub statistics: Statistics,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    #[serde(default)]
    pub album_count: i64,
    #[serde(default)]
    pub track_count: i64,
    #[serde(default)]
    pub track_file_count: i64,
    #[serde(default)]
    pub size_on_disk: i64,
    #[serde(default)]
    pub percent_of_tracks: f64,
}
