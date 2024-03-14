use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverseerrRequest {
    pub page_info: PageInfo,
    pub results: Vec<Result>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub pages: i64,
    pub page_size: i64,
    pub results: i64,
    pub page: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub id: i64,
    pub status: i64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub is4k: bool,
    pub server_id: i64,
    pub profile_id: i64,
    pub root_folder: String,
    pub language_profile_id: Option<i64>,
    pub tags: Vec<Value>,
    pub is_auto_request: bool,
    pub media: Media,
    pub seasons: Vec<Season>,
    pub modified_by: ModifiedBy,
    pub requested_by: RequestedBy,
    pub season_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub download_status: Vec<Value>,
    pub download_status4k: Vec<Value>,
    pub id: i64,
    pub media_type: String,
    pub tmdb_id: i64,
    pub tvdb_id: Option<i64>,
    pub imdb_id: Value,
    pub status: i64,
    pub status4k: i64,
    pub created_at: String,
    pub updated_at: String,
    pub last_season_change: String,
    pub media_added_at: Value,
    pub service_id: i64,
    pub service_id4k: Value,
    pub external_service_id: i64,
    pub external_service_id4k: Value,
    pub external_service_slug: String,
    pub external_service_slug4k: Value,
    pub rating_key: Value,
    pub rating_key4k: Value,
    pub service_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub id: i64,
    pub season_number: i64,
    pub status: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiedBy {
    pub permissions: i64,
    pub id: i64,
    pub email: String,
    pub plex_username: String,
    pub username: Value,
    pub recovery_link_expiration_date: Value,
    pub user_type: i64,
    pub plex_id: i64,
    pub avatar: String,
    pub movie_quota_limit: Value,
    pub movie_quota_days: Value,
    pub tv_quota_limit: Value,
    pub tv_quota_days: Value,
    pub created_at: String,
    pub updated_at: String,
    pub request_count: i64,
    pub display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestedBy {
    pub permissions: i64,
    pub id: i64,
    pub email: String,
    pub plex_username: String,
    pub username: Value,
    pub recovery_link_expiration_date: Value,
    pub user_type: i64,
    pub plex_id: i64,
    pub avatar: String,
    pub movie_quota_limit: Value,
    pub movie_quota_days: Value,
    pub tv_quota_limit: Value,
    pub tv_quota_days: Value,
    pub created_at: String,
    pub updated_at: String,
    pub request_count: i64,
    pub display_name: String,
}
