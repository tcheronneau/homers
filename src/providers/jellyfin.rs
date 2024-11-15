use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::providers::structs::jellyfin::Session;
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

    pub async fn get_sessions(&self) -> Result<Vec<Session>, ProviderError> {
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
        let sessions: Vec<Session> = match response.json().await {
            Ok(sessions) => sessions,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Jellyfin,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(sessions)
    }
}
