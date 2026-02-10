use log::error;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::readarr::Author;
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ReadarrAuthor {
    pub name: String,
    pub monitored: bool,
    pub book_file_count: i64,
}
impl std::fmt::Display for ReadarrAuthor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: monitored: {}, book_file_count: {}",
            self.name, self.monitored, self.book_file_count
        )
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Readarr {
    #[serde(skip)]
    pub name: String,
    pub address: String,
    #[serde(rename = "apikey", skip_serializing)]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
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
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Readarr {
            name: name.to_string(),
            address: format!("{}/api/v1", address),
            api_key: api_key.to_string(),
            client,
        })
    }
    async fn get_authors(&self) -> Result<Vec<Author>, ProviderError> {
        let url = format!("{}/author", self.address);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Readarr,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let authors: Vec<Author> = match response.json().await {
            Ok(authors) => authors,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Readarr,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(authors)
    }
    pub async fn get_readarr_authors(&self) -> Vec<ReadarrAuthor> {
        let authors = match self.get_authors().await {
            Ok(authors) => authors,
            Err(e) => {
                error!("Failed to get readarr authors: {:?}", e);
                Vec::new()
            }
        };

        authors
            .into_iter()
            .map(|author| ReadarrAuthor {
                name: author.author_name.clone(),
                monitored: author.monitored,
                book_file_count: author.statistics.book_file_count,
            })
            .collect::<Vec<ReadarrAuthor>>()
    }
}
