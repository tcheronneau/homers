use chrono::{format::strftime::StrftimeItems, Duration, Local};
use log::{debug, error};
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::readarr;
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Readarr {
    #[serde(skip)]
    pub name: String,
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ReadarrEpisode {
    pub sxe: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub serie: String,
    pub air_date: String,
    #[serde(rename = "hasFile")]
    pub has_file: bool,
}
impl std::fmt::Display for ReadarrEpisode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {} - {} - {}",
            self.serie, self.sxe, self.title, self.air_date, self.has_file
        )
    }
}

impl Readarr {
    pub fn new(name: &str, address: &str, api_key: &str) -> Result<Readarr, ProviderError> {
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = match header::HeaderValue::from_str(api_key) {
            Ok(header_api_key) => header_api_key,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Readarr,
                    ProviderErrorKind::HeaderError,
                    &format!("{:?}", e),
                ));
            }
        };
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Readarr {
            name: name.to_string(),
            address: address.to_string(),
            api_key: api_key.to_string(),
            client,
        })
    }
}
