use log::{debug, error};
use reqwest;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::plex::PlexResponse;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Plex {
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
            address: address.to_string(),
            token: token.to_string(),
            client,
        })
    }
    pub async fn get_history(&self) -> anyhow::Result<PlexResponse> {
        let url = format!("{}/status/sessions/history/all", self.address);
        debug!("Requesting history from {}", url);
        let response = self.client.get(&url).send().await?;
        let history = response.json::<PlexResponse>().await?;
        Ok(history)
    }
}
