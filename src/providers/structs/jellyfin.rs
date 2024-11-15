use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
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
    pub posititon_ticks: Option<i64>,
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
}
