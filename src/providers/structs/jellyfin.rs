use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionResponse {
    pub play_state: PlayState,
    pub user_name: String,
    pub device_type: String,
    pub client: String,
    pub now_playing_item: NowPlayingItem,
    pub transcoding_info: TranscodingInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayState {
    pub posititon_ticks: i64,
    pub is_paused: bool,
    pub play_method: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TranscodingInfo {
    pub is_direct_video: bool,
    pub is_direct_audio: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NowPlayingItem {
    pub name: String,
    pub run_time_ticks: i64,
}
