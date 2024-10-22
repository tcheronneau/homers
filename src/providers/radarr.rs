use log::error;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::radarr::Movie;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RadarrMovie {
    pub title: String,
    #[serde(rename = "hasFile")]
    pub has_file: bool,
    pub monitored: bool,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub missing_available: bool,
}
impl std::fmt::Display for RadarrMovie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: has_file: {}, monitored: {}, is_available: {}",
            self.title, self.has_file, self.monitored, self.is_available
        )
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Radarr {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
}
impl Radarr {
    pub fn new(address: &str, api_key: &str) -> anyhow::Result<Radarr> {
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = header::HeaderValue::from_str(&api_key).unwrap();
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Radarr {
            address: format!("{}/api/v3", address),
            api_key: api_key.to_string(),
            client,
        })
    }
    async fn get_movies(&self) -> anyhow::Result<Vec<Movie>> {
        let url = format!("{}/movie", self.address);
        let response = self.client.get(&url).send().await?;
        let movies: Vec<Movie> = match response.json().await {
            Ok(movies) => movies,
            Err(e) => {
                anyhow::bail!("Failed to parse radarr get_movies response: {:?}", e);
            }
        };
        Ok(movies)
    }
    pub async fn get_radarr_movies(&self) -> Vec<RadarrMovie> {
        let movies = match self.get_movies().await {
            Ok(movies) => movies,
            Err(e) => {
                error!("Failed to get radarr movies: {:?}", e);
                Vec::new()
            }
        };
        let mut radarr_movies = Vec::new();
        for movie in movies {
            radarr_movies.push(RadarrMovie {
                title: movie.title.clone(),
                has_file: movie.has_file,
                monitored: movie.monitored,
                is_available: movie.is_available,
                missing_available: self.set_missing_movies(&movie),
            });
        }
        radarr_movies
    }
    fn set_missing_movies(&self, movie: &Movie) -> bool {
        if !movie.has_file && movie.is_available {
            true
        } else {
            false
        }
    }
}
