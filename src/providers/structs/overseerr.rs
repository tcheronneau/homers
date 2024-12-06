use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub results: Vec<Result>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub status: i64,
    pub created_at: String,
    pub is_auto_request: bool,
    pub media: Media,
    pub requested_by: RequestedBy,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: i64,
    pub media_type: String,
    pub tmdb_id: i64,
    pub status: i64,
    pub created_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestedBy {
    pub permissions: i64,
    pub id: i64,
    #[serde(default)]
    pub plex_username: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub original_title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tv {
    pub name: String,
}
