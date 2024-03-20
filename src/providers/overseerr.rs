use reqwest::header;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::sync::{Mutex, Once,Arc};
use anyhow::Context;

use crate::providers::structs::overseerr;

lazy_static! {
    static ref API_KEY: Mutex<Option<Arc<String>>> = Mutex::new(None);
    static ref INIT: Once = Once::new();
}

fn initialize_api_key(api_key: String) {
    INIT.call_once(|| {
        *API_KEY.lock().unwrap() = Some(Arc::new(api_key));
    });
}

fn get_api_key() -> Arc<String> {
    INIT.call_once(|| {
        eprintln!("API key not initialized!");
        std::process::exit(1);
    });

    Arc::clone(API_KEY.lock().unwrap().as_ref().unwrap())
}

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
    client: Option<reqwest::blocking::Client>,
}
impl Overseerr {
    pub fn new(address: String, api_key: String, requests: i64) -> anyhow::Result<Overseerr> {
        let mut headers = header::HeaderMap::new();
        initialize_api_key(api_key.clone());
        let mut header_api_key = header::HeaderValue::from_str(&*get_api_key()).unwrap();
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        headers.insert("Content-Type", header::HeaderValue::from_static("application/json"));
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create Overseerr client ")?;
        Ok(Overseerr {
            address,
            api_key,
            requests: Some(requests),
            client: Some(client),
        })
    }
    fn get_requests(&self) -> anyhow::Result<Vec<overseerr::Result>> {
        let url = format!("{}/api/v1/request", self.address);
        let response = self.client
            .as_ref()
            .context("Failed to get client")?
            .get(&url)
            .query(&[("sort", "added")])
            .query(&[("take", self.requests.unwrap().to_string())])
            .send()
            .context("Failed to get requests")?;
        let requests = response.json::<overseerr::Request>().context("Failed to parse get_requests")?;
        Ok(requests.results)
        //Ok(Vec::new())
    }
    pub fn get_overseerr_requests(&self) -> anyhow::Result<Vec<OverseerrRequest>> {
        let requests = self.get_requests()?;
        let mut overseerr_requests = Vec::new();
        for request in requests {
            let media_title = self.get_media_title(&request.media.media_type, request.media.tmdb_id)?;
            let overseerr_request = OverseerrRequest {
                media_type: request.media.media_type.clone(),
                media_id: request.media.id,
                status: request.status,
                requested_by: self.get_username(request.clone())?, 
                media_status: request.media.status,
                media_title,
                requested_at: request.created_at,
            };
            overseerr_requests.push(overseerr_request);
        }
        Ok(overseerr_requests)
    }
    fn get_username(&self, request: overseerr::Result) -> anyhow::Result<String> {
        match request.requested_by.username {
            Some(username) => Ok(username),
            None => match request.requested_by.plex_username {
                Some(username) => Ok(username),
                None => Ok("Unknown".to_string()),
            }
        }
    }
    fn get_media_title(&self, media_type: &str, media_id: i64) -> anyhow::Result<String> {
        let url = format!("{}/api/v1/{}/{}", self.address, media_type, media_id);
        let response = self.client
            .as_ref()
            .context("Failed to get client")?
            .get(&url)
            .send()
            .context("Failed to get media title")?;
        match media_type {
            "movie" => {
                let movie: overseerr::Movie = response.json().context("Failed to parse movie")?;
                match movie.original_title {
                    Some(title) => Ok(title),
                    None => Ok("Unknown".to_string()),
                }
            }
            "tv" => {
                let show: overseerr::Tv = response.json().context("Failed to parse show")?;
                Ok(show.name)
            }
            _ => Ok("Unknown".to_string()),
        }
    }
}
