use ipgeolocate::{Locator, Service};
use log::error;
use reqwest;
use serde::{Deserialize, Serialize};

use crate::providers::structs::tautulli;
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Tautulli {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(default)]
    api_url: String,
    #[serde(skip)]
    client: reqwest::Client,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TautulliLocation {
    pub city: String,
    pub country: String,
    pub ip_address: String,
    pub latitude: String,
    pub longitude: String,
}

#[derive(Debug)]
pub struct SessionSummary {
    pub user: String,
    pub title: String,
    pub state: String,
    pub progress: String,
    pub quality: String,
    pub quality_profile: String,
    pub video_stream: String,
    pub media_type: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub location: TautulliLocation,
}
impl std::fmt::Display for SessionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.media_type == "episode" {
            write!(
                f,
                "User {} is watching {} S{:02}E{:02}. Currently the play is {} and {}% is watched",
                self.user,
                self.title,
                self.season_number.as_ref().unwrap(),
                self.episode_number.as_ref().unwrap(),
                self.state,
                self.progress
            )
        } else {
            write!(f, "User {} is watching {} in quality {} stream quality {} on {}. Currently the play is {} and {}% is watched", self.user, self.title,self.quality, self.quality_profile, self.video_stream, self.state, self.progress)
        }
    }
}

impl Tautulli {
    pub fn new(address: &str, api_key: &str) -> Result<Tautulli, ProviderError> {
        let api_url = format!("{}/api/v2?apikey={}&cmd=", address, api_key);
        let client = reqwest::Client::builder().build()?;
        Ok(Tautulli {
            api_key: api_key.to_string(),
            address: address.to_string(),
            api_url,
            client,
        })
    }
    pub async fn get(&self, command: &str) -> Result<tautulli::TautulliData, ProviderError> {
        let url = format!("{}{}", self.api_url, command);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Tautulli,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let tautulli: tautulli::TautulliResponse = match response.json().await {
            Ok(tautulli) => tautulli,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Tautulli,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(tautulli.response.data)
    }
    pub async fn get_libraries(&self) -> Vec<tautulli::Library> {
        let get_libraries = match self.get("get_libraries").await {
            Ok(libraries) => libraries,
            Err(e) => {
                error!("Failed to get libraries: {}", e);
                return Vec::new();
            }
        };
        let libraries: Vec<tautulli::Library> = get_libraries.into();
        libraries
    }
    async fn get_ip_info(&self, ip: &str) -> Result<TautulliLocation, ProviderError> {
        let service = Service::IpApi;
        match Locator::get(ip, service).await {
            Ok(location) => Ok(TautulliLocation {
                city: location.city,
                country: location.country,
                ip_address: ip.to_string(),
                latitude: location.latitude,
                longitude: location.longitude,
            }),
            Err(_) => Ok(TautulliLocation {
                city: "Unknown".to_string(),
                country: "Unknown".to_string(),
                ip_address: ip.to_string(),
                latitude: "0.0".to_string(),
                longitude: "0.0".to_string(),
            }),
        }
    }
    pub async fn get_session_summary(&self) -> Vec<SessionSummary> {
        let get_activities = match self.get("get_activity").await {
            Ok(activities) => activities,
            Err(e) => {
                error!("Failed to get activities: {}", e);
                return Vec::new();
            }
        };
        let activity: tautulli::Activity = get_activities.into();
        let mut session_summaries = Vec::new();
        for session in &activity.sessions {
            let location = match self.get_ip_info(&session.ip_address).await {
                Ok(location) => location,
                Err(e) => {
                    error!("Failed to get location: {}", e);
                    TautulliLocation {
                        city: "Unknown".to_string(),
                        country: "Unknown".to_string(),
                        ip_address: session.ip_address_public.clone(),
                        latitude: "0.0".to_string(),
                        longitude: "0.0".to_string(),
                    }
                }
            };
            let session_summary = if session.media_type == "episode" {
                SessionSummary {
                    user: session.user.clone(),
                    title: session.grandparent_title.clone(),
                    state: session.state.clone(),
                    progress: session.progress_percent.clone(),
                    quality: session.video_full_resolution.clone(),
                    quality_profile: session.quality_profile.clone(),
                    video_stream: session.video_decision.clone(),
                    media_type: session.media_type.clone(),
                    season_number: Some(session.parent_media_index.clone()),
                    episode_number: Some(session.media_index.clone()),
                    location,
                }
            } else {
                SessionSummary {
                    user: session.user.clone(),
                    title: session.title.clone(),
                    state: session.state.clone(),
                    progress: session.progress_percent.clone(),
                    quality: session.video_full_resolution.clone(),
                    quality_profile: session.quality_profile.clone(),
                    video_stream: session.video_decision.clone(),
                    media_type: session.media_type.clone(),
                    season_number: None,
                    episode_number: None,
                    location,
                }
            };
            session_summaries.push(session_summary);
        }
        session_summaries
    }
}
