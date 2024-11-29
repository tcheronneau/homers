use crate::providers::structs::AsyncFrom;
use log::error;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::jellyfin::{JellyfinLibraryCounts, SessionResponse};
use crate::providers::structs::{Session, User};
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Jellyfin {
    #[serde(skip)]
    pub name: String,
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
}

impl Jellyfin {
    pub fn new(name: &str, address: &str, api_key: &str) -> Result<Jellyfin, ProviderError> {
        let mut headers = header::HeaderMap::new();
        let header_str = format!("MediaBrowser Token=\"{}\"", api_key);
        let mut header_api_key = match header::HeaderValue::from_str(&header_str) {
            Ok(header_api_key) => header_api_key,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Jellyfin,
                    ProviderErrorKind::HeaderError,
                    &format!("{:?}", e),
                ));
            }
        };
        header_api_key.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, header_api_key);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Jellyfin {
            name: name.to_string(),
            address: address.to_string(),
            api_key: api_key.to_string(),
            client,
        })
    }

    pub async fn get_library_counts(&self) -> Result<JellyfinLibraryCounts, ProviderError> {
        let url = format!("{}/Items/Counts", self.address);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Jellyfin,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let library_counts: JellyfinLibraryCounts = match response.json().await {
            Ok(library_counts) => library_counts,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Jellyfin,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(library_counts)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>, ProviderError> {
        let url = format!("{}/Sessions", self.address);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Jellyfin,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let sessions: Vec<SessionResponse> = match response.json().await {
            Ok(sessions) => sessions,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Jellyfin,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        let mut jelly_sessions: Vec<Session> = Vec::new();
        for session in sessions {
            let session = Session::from_async(session).await;
            jelly_sessions.push(session);
        }
        Ok(jelly_sessions)
    }
    pub async fn get_current_sessions(&self) -> Vec<Session> {
        match self.get_sessions().await {
            Ok(sessions) => sessions,
            Err(e) => {
                error!("Failed to get sessions: {}", e);
                Vec::new()
            }
        }
    }
    pub async fn get_users(&self) -> Vec<User> {
        let url = format!("{}/Users", self.address);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to get users: {}", e);
                return Vec::new();
            }
        };
        let users: Vec<User> = match response.json().await {
            Ok(users) => users,
            Err(e) => {
                error!("Failed to parse users: {}", e);
                return Vec::new();
            }
        };
        users
    }
}
