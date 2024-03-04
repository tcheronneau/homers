use serde::{Serialize, Deserialize};
use serde_json::Value;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub app_name: String,
    pub instance_name: String,
    pub version: String,
    pub build_time: String,
    pub is_debug: bool,
    pub is_production: bool,
    pub is_admin: bool,
    pub is_user_interactive: bool,
    pub startup_path: String,
    pub app_data: String,
    pub os_name: String,
    #[serde(default)]
    pub os_version: Option<String>,
    pub is_net_core: bool,
    pub is_linux: bool,
    pub is_osx: bool,
    pub is_windows: bool,
    pub is_docker: bool,
    pub mode: String,
    pub branch: String,
    pub authentication: String,
    #[serde(default)]
    pub sqlite_version: Option<SqliteVersion>,
    pub migration_version: i64,
    pub url_base: String,
    pub runtime_version: String,
    pub runtime_name: String,
    pub start_time: String,
    #[serde(default)]
    pub package_version: Option<String>,
    #[serde(default)]
    pub package_author: Option<String>,
    #[serde(default)]
    pub package_update_mechanism: Option<String>,
    #[serde(default)]
    pub package_update_mechanism_message: Option<String>,
    #[serde(default)]
    pub database_version: String,
    pub database_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteVersion {
    pub major: i64,
    pub minor: i64,
    pub build: i64,
    pub revision: i64,
    pub major_revision: i64,
    pub minor_revision: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVersion {
    pub major: i64,
    pub minor: i64,
    pub build: i64,
    pub revision: i64,
    pub major_revision: i64,
    pub minor_revision: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseVersion {
    pub major: i64,
    pub minor: i64,
    pub build: i64,
    pub revision: i64,
    pub major_revision: i64,
    pub minor_revision: i64,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub series_id: i64,
    pub tvdb_id: i64,
    pub episode_file_id: i64,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub air_date: String,
    pub air_date_utc: String,
    pub runtime: i64,
    pub overview: Option<String>,
    pub has_file: bool,
    pub monitored: bool,
    pub unverified_scene_numbering: bool,
    pub grabbed: bool,
    pub id: i64,
    pub series: Series,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeFile {
    pub series_id: i64,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub air_date: String,
    pub air_date_utc: String,
    pub overview: Option<String>,
    pub has_file: bool,
    pub monitored: bool,
    pub unverified_scene_numbering: bool,
    pub grabbed: bool,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub series_id: i64,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub air_date: String,
    pub air_date_utc: String,
    pub overview: Option<String>,
    pub has_file: bool,
    pub monitored: bool,
    pub unverified_scene_numbering: bool,
    pub grabbed: bool,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub title: String,
    pub sort_title: String,
    pub status: String,
    pub ended: bool,
    pub overview: String,
    pub network: String,
    pub air_time: String,
    pub images: Vec<Image>,
    pub original_language: OriginalLanguage,
    pub seasons: Vec<Season>,
    pub year: i64,
    pub path: String,
    pub quality_profile_id: i64,
    pub season_folder: bool,
    pub monitored: bool,
    pub monitor_new_items: String,
    pub use_scene_numbering: bool,
    pub runtime: i64,
    pub tvdb_id: i64,
    pub tv_rage_id: i64,
    pub tv_maze_id: i64,
    pub first_aired: String,
    pub last_aired: String,
    pub series_type: String,
    pub clean_title: String,
    pub imdb_id: String,
    pub title_slug: String,
    pub certification: String,
    pub genres: Vec<String>,
    pub tags: Vec<Value>,
    pub added: String,
    pub ratings: Ratings,
    pub language_profile_id: i64,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub cover_type: String,
    pub remote_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalLanguage {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub season_number: i64,
    pub monitored: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratings {
    pub votes: i64,
    pub value: i64,
}
