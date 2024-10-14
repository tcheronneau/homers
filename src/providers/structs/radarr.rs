use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub title: String,
    #[serde(rename = "originalTitle")]
    pub original_title: String,
    #[serde(rename = "originalLanguage")]
    pub original_language: OriginalLanguage,
    #[serde(rename = "alternateTitles")]
    pub alternate_titles: Vec<AlternateTitle>,
    #[serde(rename = "secondaryYearSourceId")]
    pub secondary_year_source_id: i64,
    #[serde(rename = "sortTitle")]
    pub sort_title: String,
    #[serde(rename = "sizeOnDisk")]
    pub size_on_disk: i64,
    pub status: String,
    pub overview: String,
    #[serde(rename = "inCinemas")]
    #[serde(default)]
    pub in_cinemas: Option<String>,
    pub images: Vec<Image>,
    pub website: String,
    pub year: i64,
    #[serde(rename = "youTubeTrailerId")]
    pub you_tube_trailer_id: String,
    pub studio: String,
    pub path: String,
    #[serde(rename = "qualityProfileId")]
    pub quality_profile_id: i64,
    #[serde(rename = "hasFile")]
    pub has_file: bool,
    pub monitored: bool,
    #[serde(rename = "minimumAvailability")]
    pub minimum_availability: String,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    #[serde(rename = "folderName")]
    pub folder_name: String,
    pub runtime: i64,
    #[serde(rename = "cleanTitle")]
    pub clean_title: String,
    #[serde(rename = "imdbId")]
    #[serde(default)]
    pub imdb_id: Option<String>,
    #[serde(rename = "tmdbId")]
    pub tmdb_id: i64,
    #[serde(rename = "titleSlug")]
    pub title_slug: String,
    #[serde(rename = "rootFolderPath")]
    pub root_folder_path: String,
    pub certification: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<Value>,
    pub added: String,
    pub ratings: Ratings,
    pub popularity: f64,
    pub statistics: Statistics,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalLanguage {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternateTitle {
    pub source_type: String,
    pub movie_metadata_id: i64,
    pub title: String,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub cover_type: String,
    pub url: String,
    pub remote_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratings {
    pub tmdb: Tmdb,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmdb {
    pub votes: i64,
    pub value: f64,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub movie_file_count: i64,
    pub size_on_disk: i64,
    pub release_groups: Vec<Value>,
}
