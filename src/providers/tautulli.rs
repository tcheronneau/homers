use reqwest;
use log::debug;
use serde::{Serialize, Deserialize};
use ipgeolocate::{Locator, Service};
use tokio::runtime::Runtime;

use crate::providers::structs::tautulli;

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
            write!(f, "User {} is watching {} S{:02}E{:02}. Currently the play is {} and {}% is watched", self.user, self.title, self.season_number.as_ref().unwrap(), self.episode_number.as_ref().unwrap(), self.state, self.progress)
        } else {
            write!(f, "User {} is watching {} in quality {} stream quality {} on {}. Currently the play is {} and {}% is watched", self.user, self.title,self.quality, self.quality_profile, self.video_stream, self.state, self.progress)
        }
    }
}

impl Tautulli {
    pub fn new(address: String, api_key: String) -> anyhow::Result<Tautulli> {
        let api_url = format!("{}/api/v2?apikey={}&cmd=", address, api_key);
        let client = reqwest::Client::builder()
            .build()?;
        Ok(Tautulli {
            api_key,
            address,
            api_url,
            client,
        })
    }
    pub async fn get(&self, command: &str) -> anyhow::Result<tautulli::TautulliData> {
        let url = format!("{}{}", self.api_url, command);
        let response = self.client
            .get(&url)
            .send()
            .await?;
        let response = response.text().await.expect("Failed to get response text");
        debug!("{}", response);
        let tautulli_response: tautulli::TautulliResponse = serde_json::from_str(&response).expect("Failed to parse JSON");
        Ok(tautulli_response.response.data)
    }
    pub async fn get_libraries(&self) -> anyhow::Result<Vec<tautulli::Library>>{
        let get_libraries = self.get("get_libraries").await?;
        let libraries: Vec<tautulli::Library> = get_libraries.into();
        Ok(libraries)
    }
    async fn get_ip_info(&self, ip: &str) -> anyhow::Result<TautulliLocation> { 
        let service = Service::IpApi;
        match Locator::get(ip, service).await {
            Ok(location) => {
                Ok(TautulliLocation {
                    city: location.city,
                    country: location.country,
                    ip_address: ip.to_string(), 
                    latitude: location.latitude,
                    longitude: location.longitude,
                })
            },
            Err(_) => {
                Ok(TautulliLocation {
                    city: "Unknown".to_string(),
                    country: "Unknown".to_string(),
                    ip_address: ip.to_string(),
                    latitude: "0.0".to_string(),
                    longitude: "0.0".to_string(),
                })
            }
        }

        
    }
    pub async fn get_session_summary(&self) -> anyhow::Result<Vec<SessionSummary>> {
        let get_activities = self.get("get_activity").await?;
        let activity: tautulli::Activity = get_activities.into();
        let session_summaries: Vec<SessionSummary> = activity.sessions.iter().map(|session| {
            let location = Runtime::new()
                .unwrap()
                .block_on(
                    self.get_ip_info(&session.ip_address)
                );
            if session.media_type == "episode" {
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
                    location: Result::expect(location, "Failed to get location"),
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
                    location: Result::expect(location, "Failed to get location"),
                }
            }
        }).collect();
        Ok(session_summaries)
    }
}
