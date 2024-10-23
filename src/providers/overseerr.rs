use anyhow::Context;
use log::error;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::overseerr;
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct OverseerrRequest {
    pub media_type: String,
    pub media_id: i64,
    pub status: i64,
    pub requested_by: String,
    pub media_status: i64,
    pub media_title: String,
    pub requested_at: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Overseerr {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    pub requests: Option<i64>,
    #[serde(skip)]
    client: reqwest::Client,
}
impl Overseerr {
    pub fn new(address: &str, api_key: &str, requests: i64) -> Result<Overseerr, ProviderError> {
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = header::HeaderValue::from_str(api_key).unwrap();
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Overseerr {
            address: address.to_string(),
            api_key: api_key.to_string(),
            requests: Some(requests),
            client,
        })
    }
    async fn get_requests(&self) -> Result<Vec<overseerr::Result>, ProviderError> {
        let url = format!("{}/api/v1/request", self.address);
        let response = match self
            .client
            .get(&url)
            .query(&[("sort", "added")])
            .query(&[("take", self.requests.unwrap().to_string())])
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Overseerr,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let requests = match response.json::<overseerr::Request>().await {
            Ok(requests) => requests,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Overseerr,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(requests.results)
        //Ok(Vec::new())
    }
    pub async fn get_overseerr_requests(&self) -> Vec<OverseerrRequest> {
        let requests = match self.get_requests().await {
            Ok(requests) => requests,
            Err(e) => {
                error!("Failed to get overseerr requests: {:?}", e);
                Vec::new()
            }
        };
        let mut overseerr_requests = Vec::new();
        for request in requests {
            let media_title = match self
                .get_media_title(&request.media.media_type, request.media.tmdb_id)
                .await
            {
                Ok(title) => title,
                Err(e) => {
                    error!("Failed to get media title: {:?}", e);
                    "Unknown".to_string()
                }
            };
            let overseerr_request = OverseerrRequest {
                media_type: request.media.media_type.clone(),
                media_id: request.media.id,
                status: request.status,
                requested_by: match self.get_username(request.clone()) {
                    Ok(username) => username,
                    Err(e) => {
                        error!("Failed to get username: {:?}", e);
                        "Unknown".to_string()
                    }
                },
                media_status: request.media.status,
                media_title,
                requested_at: request.created_at,
            };
            overseerr_requests.push(overseerr_request);
        }
        overseerr_requests
    }
    fn get_username(&self, request: overseerr::Result) -> anyhow::Result<String> {
        match request.requested_by.username {
            Some(username) => Ok(username),
            None => match request.requested_by.plex_username {
                Some(username) => Ok(username),
                None => Ok("Unknown".to_string()),
            },
        }
    }
    async fn get_media_title(
        &self,
        media_type: &str,
        media_id: i64,
    ) -> Result<String, ProviderError> {
        let url = format!("{}/api/v1/{}/{}", self.address, media_type, media_id);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Overseerr,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        match media_type {
            "movie" => {
                let movie: overseerr::Movie =
                    match response.json().await.context("Failed to parse movie") {
                        Ok(movie) => movie,
                        Err(e) => {
                            return Err(ProviderError::new(
                                Provider::Overseerr,
                                ProviderErrorKind::ParseError,
                                &format!("{:?}", e),
                            ));
                        }
                    };
                match movie.original_title {
                    Some(title) => Ok(title),
                    None => Ok("Unknown".to_string()),
                }
            }
            "tv" => {
                let show: overseerr::Tv =
                    match response.json().await.context("Failed to parse show") {
                        Ok(show) => show,
                        Err(e) => {
                            return Err(ProviderError::new(
                                Provider::Overseerr,
                                ProviderErrorKind::ParseError,
                                &format!("{:?}", e),
                            ));
                        }
                    };
                Ok(show.name)
            }
            _ => Ok("Unknown".to_string()),
        }
    }
}
