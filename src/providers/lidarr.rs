use log::error;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::lidarr::Artist;
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LidarrArtist {
    pub name: String,
    pub monitored: bool,
    pub track_file_count: i64,
}

impl std::fmt::Display for LidarrArtist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: monitored: {}, track_file_count: {}",
            self.name, self.monitored, self.track_file_count
        )
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Lidarr {
    #[serde(skip)]
    pub name: String,
    pub address: String,
    #[serde(rename = "apikey", skip_serializing)]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
}

impl Lidarr {
    pub fn new(name: &str, address: &str, api_key: &str) -> Result<Lidarr, ProviderError> {
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = match header::HeaderValue::from_str(api_key) {
            Ok(header_api_key) => header_api_key,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Lidarr,
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
        Ok(Lidarr {
            name: name.to_string(),
            address: format!("{}/api/v1", address),
            api_key: api_key.to_string(),
            client,
        })
    }

    async fn get_artists(&self) -> Result<Vec<Artist>, ProviderError> {
        let url = format!("{}/artist", self.address);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Lidarr,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let artists: Vec<Artist> = match response.json().await {
            Ok(artists) => artists,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Lidarr,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(artists)
    }

    pub async fn get_lidarr_artists(&self) -> Vec<LidarrArtist> {
        let artists = match self.get_artists().await {
            Ok(artists) => artists,
            Err(e) => {
                error!("Failed to get lidarr artists: {:?}", e);
                Vec::new()
            }
        };

        artists
            .into_iter()
            .map(|artist| LidarrArtist {
                name: artist.artist_name.clone(),
                monitored: artist.monitored,
                track_file_count: artist.statistics.track_file_count,
            })
            .collect::<Vec<LidarrArtist>>()
    }
}
