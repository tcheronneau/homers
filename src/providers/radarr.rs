use reqwest::header;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::sync::{Mutex, Once,Arc};
use log::debug;

use crate::providers::structs::radarr::Movie;

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
pub struct RadarrMovie {
    pub title: String,
    #[serde(rename = "hasFile")]
    pub has_file: bool,
    pub monitored: bool,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub missing_available: bool,
}
impl std::fmt::Display for RadarrMovie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: has_file: {}, monitored: {}, is_available: {}", self.title, self.has_file, self.monitored, self.is_available)
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Radarr {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(skip)]
    client: Option<reqwest::blocking::Client>,
}
impl Radarr {
    pub fn new(address: String, api_key: String) -> anyhow::Result<Radarr> {
        let mut headers = header::HeaderMap::new();
        initialize_api_key(api_key.clone());
        let mut header_api_key = header::HeaderValue::from_str(&*get_api_key()).unwrap();
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        headers.insert("Accept", header::HeaderValue::from_static("application/json"));
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Radarr {
            address: format!("{}/api/v3", address),
            api_key,
            client: Some(client),
        })
    }
    fn get_movies(&self) -> anyhow::Result<Vec<Movie>> {
        let url = format!("{}/movie", self.address);
        let response = self.client
            .as_ref()
            .expect("Failed to create radarr client")
            .get(&url)
            .send()?;
        let movies: Vec<Movie> = response.json()?;
        Ok(movies)
    }
    pub fn get_radarr_movies(&self) -> anyhow::Result<Vec<RadarrMovie>> {
        let movies = self.get_movies()?;
        let mut radarr_movies = Vec::new();
        for movie in movies {
            radarr_movies.push(RadarrMovie {
                title: movie.title.clone(),
                has_file: movie.has_file,
                monitored: movie.monitored,
                is_available: movie.is_available,
                missing_available: self.set_missing_movies(&movie),
            });
        }
        Ok(radarr_movies)
    }
    fn set_missing_movies(&self, movie: &Movie) -> bool {
        if !movie.has_file && movie.is_available {
            true
        } else {
            false
        }
    }
}
