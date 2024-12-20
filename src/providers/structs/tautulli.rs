use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct TautulliResponse {
    pub response: ActivityResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityResponse {
    pub result: String,
    pub message: Option<String>,
    pub data: TautulliData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TautulliData {
    Activity(Activity),
    Libraries(Vec<Library>),
}
impl Into<Activity> for TautulliData {
    fn into(self) -> Activity {
        match self {
            TautulliData::Activity(activity) => activity,
            _ => panic!("TautulliData is not Activity"),
        }
    }
}
impl Into<Vec<Library>> for TautulliData {
    fn into(self) -> Vec<Library> {
        match self {
            TautulliData::Libraries(libraries) => libraries,
            _ => panic!("TautulliData is not Libraries"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub stream_count: String,
    pub sessions: Vec<Session>,
    pub stream_count_direct_play: i64,
    pub stream_count_direct_stream: i64,
    pub stream_count_transcode: i64,
    pub total_bandwidth: i64,
    pub lan_bandwidth: i64,
    pub wan_bandwidth: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_key: String,
    pub media_type: String,
    pub view_offset: String,
    pub progress_percent: String,
    pub quality_profile: String,
    pub synced_version_profile: String,
    pub optimized_version_profile: String,
    pub user: String,
    pub channel_stream: i64,
    pub section_id: String,
    pub library_name: String,
    pub rating_key: String,
    pub parent_rating_key: String,
    pub grandparent_rating_key: String,
    pub title: String,
    pub parent_title: String,
    pub grandparent_title: String,
    pub original_title: String,
    pub sort_title: String,
    pub edition_title: String,
    pub media_index: String,
    pub parent_media_index: String,
    pub studio: String,
    pub content_rating: String,
    pub summary: String,
    pub tagline: String,
    pub rating: String,
    pub rating_image: String,
    pub audience_rating: String,
    pub audience_rating_image: String,
    pub user_rating: String,
    pub duration: String,
    pub year: String,
    pub parent_year: String,
    pub grandparent_year: String,
    pub thumb: String,
    pub parent_thumb: String,
    pub grandparent_thumb: String,
    pub art: String,
    pub banner: String,
    pub originally_available_at: String,
    pub added_at: String,
    pub updated_at: String,
    pub last_viewed_at: String,
    pub guid: String,
    pub parent_guid: String,
    pub grandparent_guid: String,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
    pub actors: Vec<String>,
    pub genres: Vec<String>,
    pub labels: Vec<Value>,
    pub collections: Vec<String>,
    pub guids: Vec<String>,
    pub markers: Vec<Marker>,
    pub parent_guids: Vec<Value>,
    pub grandparent_guids: Vec<Value>,
    pub full_title: String,
    pub children_count: i64,
    pub live: i64,
    pub id: String,
    pub container: String,
    pub bitrate: String,
    pub height: String,
    pub width: String,
    pub aspect_ratio: String,
    pub video_codec: String,
    pub video_resolution: String,
    pub video_full_resolution: String,
    pub video_framerate: String,
    pub video_profile: String,
    pub audio_codec: String,
    pub audio_channels: String,
    pub audio_channel_layout: String,
    pub audio_profile: String,
    pub optimized_version: i64,
    pub channel_call_sign: String,
    pub channel_identifier: String,
    pub channel_thumb: String,
    pub file: String,
    pub file_size: String,
    pub indexes: i64,
    pub selected: i64,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub video_codec_level: String,
    pub video_bitrate: String,
    pub video_bit_depth: String,
    pub video_chroma_subsampling: String,
    pub video_color_primaries: String,
    pub video_color_range: String,
    pub video_color_space: String,
    pub video_color_trc: String,
    pub video_dynamic_range: String,
    pub video_frame_rate: String,
    pub video_ref_frames: String,
    pub video_height: String,
    pub video_width: String,
    pub video_language: String,
    pub video_language_code: String,
    pub video_scan_type: String,
    pub audio_bitrate: String,
    pub audio_bitrate_mode: String,
    pub audio_sample_rate: String,
    pub audio_language: String,
    pub audio_language_code: String,
    pub subtitle_codec: String,
    pub subtitle_container: String,
    pub subtitle_format: String,
    pub subtitle_forced: i64,
    pub subtitle_location: String,
    pub subtitle_language: String,
    pub subtitle_language_code: String,
    pub row_id: i64,
    pub user_id: i64,
    pub username: String,
    pub friendly_name: String,
    pub user_thumb: String,
    pub email: String,
    pub is_active: i64,
    pub is_admin: i64,
    pub is_home_user: i64,
    pub is_allow_sync: i64,
    pub is_restricted: i64,
    pub do_notify: i64,
    pub keep_history: i64,
    pub deleted_user: i64,
    pub allow_guest: i64,
    pub shared_libraries: Vec<String>,
    pub last_seen: Value,
    pub ip_address: String,
    pub ip_address_public: String,
    pub device: String,
    pub platform: String,
    pub platform_name: String,
    pub platform_version: String,
    pub product: String,
    pub product_version: String,
    pub profile: String,
    pub player: String,
    pub machine_id: String,
    pub state: String,
    pub local: i64,
    pub relayed: i64,
    pub secure: i64,
    pub session_id: String,
    pub bandwidth: String,
    pub location: String,
    pub transcode_key: String,
    pub transcode_throttled: i64,
    pub transcode_progress: i64,
    pub transcode_speed: String,
    pub transcode_audio_channels: String,
    pub transcode_audio_codec: String,
    pub transcode_video_codec: String,
    pub transcode_width: String,
    pub transcode_height: String,
    pub transcode_container: String,
    pub transcode_protocol: String,
    pub transcode_min_offset_available: i64,
    pub transcode_max_offset_available: i64,
    pub transcode_hw_requested: i64,
    pub transcode_hw_decode: String,
    pub transcode_hw_decode_title: String,
    pub transcode_hw_encode: String,
    pub transcode_hw_encode_title: String,
    pub transcode_hw_full_pipeline: i64,
    pub audio_decision: String,
    pub video_decision: String,
    pub subtitle_decision: String,
    pub throttled: String,
    pub transcode_hw_decoding: i64,
    pub transcode_hw_encoding: i64,
    pub stream_container: String,
    pub stream_bitrate: String,
    pub stream_aspect_ratio: String,
    pub stream_video_framerate: String,
    pub stream_video_resolution: String,
    pub stream_duration: String,
    pub stream_container_decision: String,
    pub optimized_version_title: String,
    pub synced_version: i64,
    pub live_uuid: String,
    pub bif_thumb: String,
    pub subtitles: i64,
    pub transcode_decision: String,
    pub container_decision: String,
    pub stream_video_full_resolution: String,
    pub stream_video_bitrate: String,
    pub stream_video_bit_depth: String,
    pub stream_video_chroma_subsampling: String,
    pub stream_video_codec: String,
    pub stream_video_codec_level: String,
    pub stream_video_color_primaries: String,
    pub stream_video_color_range: String,
    pub stream_video_color_space: String,
    pub stream_video_color_trc: String,
    pub stream_video_dynamic_range: String,
    pub stream_video_height: String,
    pub stream_video_width: String,
    pub stream_video_ref_frames: String,
    pub stream_video_language: String,
    pub stream_video_language_code: String,
    pub stream_video_scan_type: String,
    pub stream_video_decision: String,
    pub stream_audio_bitrate: String,
    pub stream_audio_bitrate_mode: String,
    pub stream_audio_channels: String,
    pub stream_audio_channel_layout: String,
    pub stream_audio_codec: String,
    pub stream_audio_sample_rate: String,
    #[serde(rename = "stream_audio_channel_layout_")]
    pub stream_audio_channel_layout2: String,
    pub stream_audio_language: String,
    pub stream_audio_language_code: String,
    pub stream_audio_decision: String,
    pub stream_subtitle_codec: String,
    pub stream_subtitle_container: String,
    pub stream_subtitle_format: String,
    pub stream_subtitle_forced: i64,
    pub stream_subtitle_location: String,
    pub stream_subtitle_language: String,
    pub stream_subtitle_language_code: String,
    pub stream_subtitle_decision: String,
    pub stream_subtitle_transient: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Marker {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub start_time_offset: i64,
    pub end_time_offset: i64,
    pub first: Option<bool>,
    #[serde(rename = "final")]
    pub final_field: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryResponse {
    #[serde(default)]
    #[serde(rename = "recordsFiltered")]
    pub records_filtered: Option<i64>,
    #[serde(default)]
    #[serde(rename = "recordsTotal")]
    pub records_total: Option<i64>,
    pub data: Vec<Library>,
    pub draw: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    #[serde(rename = "section_id")]
    pub section_id: String,
    #[serde(rename = "section_name")]
    pub section_name: String,
    #[serde(rename = "section_type")]
    pub section_type: String,
    pub agent: String,
    pub thumb: String,
    pub art: String,
    pub count: String,
    #[serde(rename = "is_active")]
    pub is_active: i64,
    #[serde(rename = "parent_count")]
    pub parent_count: Option<String>,
    #[serde(rename = "child_count")]
    pub child_count: Option<String>,
}
impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.section_type == "show" {
            write!(
                f,
                "Library {} has {} shows {} seasons and {} episodes",
                self.section_name,
                self.count,
                self.parent_count.as_ref().unwrap(),
                self.child_count.as_ref().unwrap()
            )
        } else if self.section_type == "movie" {
            write!(f, "Library {} has {} movies", self.section_name, self.count)
        } else if self.section_type == "artist" {
            write!(
                f,
                "Library {} has {} artists {} albums {} tracks",
                self.section_name,
                self.count,
                self.parent_count.as_ref().unwrap(),
                self.child_count.as_ref().unwrap()
            )
        } else {
            write!(f, "Library {} has {} items", self.section_name, self.count)
        }
    }
}
