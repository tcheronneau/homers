use log::error;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::radarr::Movie;
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

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
    #[serde(skip)]
    pub name: String,
    pub address: String,
    #[serde(rename = "apikey", skip_serializing)]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
}
impl Radarr {
    pub fn new(name: &str, address: &str, api_key: &str) -> Result<Radarr, ProviderError> {
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = match header::HeaderValue::from_str(api_key) {
            Ok(header_api_key) => header_api_key,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Radarr,
                    ProviderErrorKind::HeaderError,
                    &format!("{:?}", e),
                ));
            }
        };
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
            name: name.to_string(),
            address: format!("{}/api/v3", address),
            api_key: api_key.to_string(),
            client,
        })
    }
    async fn get_movies(&self) -> Result<Vec<Movie>, ProviderError> {
        let url = format!("{}/movie", self.address);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Radarr,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let movies: Vec<Movie> = match response.json().await {
            Ok(movies) => movies,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Radarr,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
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
        
        movies
            .into_iter()
            .map(|movie| RadarrMovie {
                title: movie.title.clone(),
                has_file: movie.has_file,
                monitored: movie.monitored,
                is_available: movie.is_available,
                missing_available: self.set_missing_movies(&movie),
            })
            .collect::<Vec<RadarrMovie>>()
    }
    fn set_missing_movies(&self, movie: &Movie) -> bool {
        !movie.has_file && movie.is_available
    }
}
