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
    pub status: RequestStatus,
    pub requested_by: String,
    pub media_status: MediaStatus,
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

#[derive(Debug, Deserialize, Clone, Serialize)]
pub enum RequestStatus {
    Pending,
    Approved,
    Declined,
}
impl From<i64> for RequestStatus {
    fn from(status: i64) -> Self {
        match status {
            1 => RequestStatus::Pending,
            2 => RequestStatus::Approved,
            3 => RequestStatus::Declined,
            _ => RequestStatus::Pending,
        }
    }
}
impl RequestStatus {
    pub fn as_f64(&self) -> f64 {
        match self {
            RequestStatus::Pending => 1.0,
            RequestStatus::Approved => 2.0,
            RequestStatus::Declined => 3.0,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            RequestStatus::Pending => "pending_approval".to_string(),
            RequestStatus::Approved => "approved".to_string(),
            RequestStatus::Declined => "declined".to_string(),
        }
    }
    pub fn _to_description(&self) -> String {
        match self {
            RequestStatus::Pending => "Overseerr request pending approval".to_string(),
            RequestStatus::Approved => "Overseerr request approved".to_string(),
            RequestStatus::Declined => "Overseerr request declined".to_string(),
        }
    }

    pub fn _get_all() -> Vec<RequestStatus> {
        vec![
            RequestStatus::Pending,
            RequestStatus::Approved,
            RequestStatus::Declined,
        ]
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub enum MediaStatus {
    Unknown,
    Pending,
    Processing,
    PartiallyAvailable,
    Available,
}
impl From<i64> for MediaStatus {
    fn from(status: i64) -> Self {
        match status {
            1 => MediaStatus::Unknown,
            2 => MediaStatus::Pending,
            3 => MediaStatus::Processing,
            4 => MediaStatus::PartiallyAvailable,
            5 => MediaStatus::Available,
            _ => MediaStatus::Unknown,
        }
    }
}
impl MediaStatus {
    pub fn as_f64(&self) -> f64 {
        match self {
            MediaStatus::Unknown => 1.0,
            MediaStatus::Pending => 2.0,
            MediaStatus::Processing => 3.0,
            MediaStatus::PartiallyAvailable => 4.0,
            MediaStatus::Available => 5.0,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            MediaStatus::Unknown => "unknown".to_string(),
            MediaStatus::Pending => "pending".to_string(),
            MediaStatus::Processing => "processing".to_string(),
            MediaStatus::PartiallyAvailable => "partially_available".to_string(),
            MediaStatus::Available => "available".to_string(),
        }
    }
    pub fn _to_description(&self) -> String {
        match self {
            MediaStatus::Unknown => "Overseerr media status unknown".to_string(),
            MediaStatus::Pending => "Overseerr media status pending".to_string(),
            MediaStatus::Processing => "Overseerr media status processing".to_string(),
            MediaStatus::PartiallyAvailable => {
                "Overseerr media status partially available".to_string()
            }
            MediaStatus::Available => "Overseerr media status available".to_string(),
        }
    }

    pub fn _get_all() -> Vec<MediaStatus> {
        vec![
            MediaStatus::Unknown,
            MediaStatus::Pending,
            MediaStatus::Processing,
            MediaStatus::PartiallyAvailable,
            MediaStatus::Available,
        ]
    }
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
        let futures_requests = requests.into_iter().map(|request| {
            let self_ref = self.clone(); // Assuming `self` implements `Clone`, so we can move it into the future.
            async move {
                // Fetch media title asynchronously
                let media_title = match self_ref
                    .get_media_title(&request.media.media_type, request.media.tmdb_id)
                    .await
                {
                    Ok(title) => title,
                    Err(e) => {
                        error!("Failed to get media title: {:?}", e);
                        "Unknown".to_string()
                    }
                };

                // Construct the OverseerrRequest
                OverseerrRequest {
                    media_type: request.media.media_type.clone(),
                    media_id: request.media.id,
                    status: request.status.into(),
                    requested_by: self_ref.get_username(&request).to_string(),
                    media_status: request.media.status.into(),
                    media_title,
                    requested_at: request.created_at,
                }
            }
        });
        let overseerr_requests: Vec<OverseerrRequest> = futures::future::join_all(futures_requests)
            .await
            .into_iter()
            .collect();
        overseerr_requests
    }
    fn get_username<'a>(&self, request: &'a overseerr::Result) -> &'a str {
        match &request.requested_by.username {
            Some(username) => username,
            None => match &request.requested_by.plex_username {
                Some(username) => username,
                None => "Unknown",
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
