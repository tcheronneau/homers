use reqwest::header;
use chrono::{Local, format::strftime::StrftimeItems, Duration};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::sync::{Mutex, Once,Arc};
use log::debug;
use anyhow::{Result, Context};

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
pub struct Overseerr {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(skip)]
    client: Option<reqwest::blocking::Client>,
}
impl Overseerr {
    pub fn new(address: String, api_key: String) -> Result<Overseerr> {
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
            client: Some(client),
        })
    }
    pub fn get_requests(&self) -> Result<Vec<overseerr::Result>> {
        let url = format!("{}/api/v1/request", self.address);
        let response = self.client
            .as_ref()
            .context("Failed to get client")?
            .get(&url)
            .query(&[("sort", "added")])
            .query(&[("take", "20")])
            .send()
            .context("Failed to get requests")?;
        let requests = response.json::<overseerr::OverseerrRequest>().context("Failed to parse get_requests")?;
        Ok(requests.results)
    }
}
