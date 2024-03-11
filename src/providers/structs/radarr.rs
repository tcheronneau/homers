use serde_json::Value;
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub title: String,
    pub original_title: String,
    pub original_language: OriginalLanguage,
    pub alternate_titles: Vec<AlternateTitle>,
    pub secondary_year_source_id: i64,
    pub sort_title: String,
    pub size_on_disk: i64,
    pub status: String,
    pub overview: String,
    pub in_cinemas: String,
    pub images: Vec<Image>,
    pub website: String,
    pub year: i64,
    pub you_tube_trailer_id: String,
    pub studio: String,
    pub path: String,
    pub quality_profile_id: i64,
    pub has_file: bool,
    pub monitored: bool,
    pub minimum_availability: String,
    pub is_available: bool,
    pub folder_name: String,
    pub runtime: i64,
    pub clean_title: String,
    pub imdb_id: String,
    pub tmdb_id: i64,
    pub title_slug: String,
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
