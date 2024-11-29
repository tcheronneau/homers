use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionResponse {
    pub play_state: PlayState,
    pub user_name: String,
    pub device_type: Option<String>,
    pub client: String,
    pub now_playing_item: Option<NowPlayingItem>,
    pub transcoding_info: Option<TranscodingInfo>,
    pub remote_end_point: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayState {
    pub position_ticks: Option<i64>,
    pub is_paused: Option<bool>,
    pub play_method: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TranscodingInfo {
    pub is_video_direct: bool,
    pub is_audio_direct: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NowPlayingItem {
    pub name: String,
    pub run_time_ticks: i64,
    #[serde(rename = "Type")]
    pub type_field: String,
    pub media_streams: Vec<MediaStream>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaStream {
    pub codec: String,
    #[serde(rename = "Type")]
    pub type_field: String,
    pub display_title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct JellyfinLibraryCounts {
    pub movie_count: i64,
    pub series_count: i64,
    pub episode_count: i64,
    pub artist_count: i64,
    pub program_count: i64,
    pub trailer_count: i64,
    pub song_count: i64,
    pub album_count: i64,
    pub music_video_count: i64,
    pub box_set_count: i64,
    pub book_count: i64,
    pub item_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub name: String,
}
