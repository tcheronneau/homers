use log::{debug, error};
use reqwest;
use reqwest::header;
use serde::{Deserialize, Serialize};

pub use crate::providers::structs::plex::SessionMetadata;
use crate::providers::structs::plex::{Metadata, PlexResponse};
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PlexViews {
    pub episodes_viewed: i64,
    pub movies_viewed: i64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Plex {
    #[serde(skip)]
    pub name: String,
    pub address: String,
    pub token: String,
    #[serde(skip)]
    pub client: reqwest::Client,
}
impl Plex {
    pub fn default() -> Plex {
        match Plex::new("default", "http://localhost:32400", "123456789") {
            Ok(plex) => plex,
            Err(e) => {
                eprintln!("Failed to create default Plex struct: {}", e);
                std::process::exit(1);
            }
        }
    }
    pub fn new(name: &str, address: &str, token: &str) -> anyhow::Result<Plex> {
        let mut headers = header::HeaderMap::new();
        let mut header_token = header::HeaderValue::from_str(&token)?;
        header_token.set_sensitive(true);
        headers.insert("X-Plex-Token", header_token);
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Plex {
            name: name.to_string(),
            address: address.to_string(),
            token: token.to_string(),
            client,
        })
    }
    async fn get_history(&self) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/status/sessions/history/all", self.address);
        debug!("Requesting history from {}", url);
        let response = self.client.get(&url).send().await?;
        let history = response.json::<PlexResponse>().await?;
        Ok(history)
    }

    async fn get_sessions(&self) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/status/sessions", self.address);
        debug!("Requesting session from {}", url);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Plex,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let session = match response.json::<PlexResponse>().await {
            Ok(session) => session,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Plex,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(session)
    }

    pub async fn get_current_sessions(&self) -> Vec<SessionMetadata> {
        let sessions = match self.get_sessions().await {
            Ok(sessions) => sessions,
            Err(e) => {
                error!("Failed to get sessions: {}", e);
                return Vec::new();
            }
        };
        let mut current_sessions: Vec<SessionMetadata> = Vec::new();
        sessions
            .media_container
            .metadata
            .iter()
            .for_each(|item| match item {
                Metadata::SessionMetadata(meta) => {
                    current_sessions.push(meta.clone());
                }
                _ => {
                    dbg!(item);
                    error!("Metadata received does not match session metadata");
                }
            });
        current_sessions
    }

    pub async fn get_views(&self) -> PlexViews {
        let history = match self.get_history().await {
            Ok(history) => history,
            Err(e) => {
                error!("Failed to get history: {}", e);
                return PlexViews {
                    episodes_viewed: 0,
                    movies_viewed: 0,
                };
            }
        };
        let mut episodes_viewed = 0;
        let mut movies_viewed = 0;
        history
            .media_container
            .metadata
            .iter()
            .for_each(|item| match item {
                Metadata::HistoryMetadata(meta) => {
                    if meta.type_field == "episode" {
                        episodes_viewed += 1;
                    } else if meta.type_field == "movie" {
                        movies_viewed += 1;
                    }
                }
                _ => {
                    error!("Metadata received does not match history metadata");
                }
            });
        PlexViews {
            episodes_viewed,
            movies_viewed,
        }
    }
}
