use reqwest;
use log::debug;
use serde::{Serialize, Deserialize};

use crate::providers::structs::tautulli;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Tautulli {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    api_url: Option<String>,
    #[serde(skip)]
    client: Option<reqwest::blocking::Client>,
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
    pub fn new(address: String, api_key: String) -> Tautulli {
        let api_url = format!("{}/api/v2?apikey={}&cmd=", address, api_key);
        let client = reqwest::blocking::Client::builder()
            .build()
            .expect("Failed to create tautulli client");
        Tautulli {
            api_key,
            address,
            api_url: Some(api_url),
            client: Some(client),
        }
    }
    pub fn get(&self, command: &str) -> anyhow::Result<tautulli::TautulliData> {
        let url = format!("{}{}", self.api_url.as_ref().unwrap(), command);
        let response = self.client
            .as_ref()
            .expect("Failed to get client")
            .get(&url)
            .send()
            .expect("Failed to send request");
        let response = response.text().expect("Failed to get response text");
        debug!("{}", response);
        let tautulli_response: tautulli::TautulliResponse = serde_json::from_str(&response).expect("Failed to parse JSON");
        Ok(tautulli_response.response.data)
    }
    pub fn get_libraries(&self) -> Vec<tautulli::Library>{
        let get_libraries = self.get("get_libraries").expect("Failed to get libraries");
        let libraries: Vec<tautulli::Library> = get_libraries.into();
        libraries
    }
    pub fn get_session_summary(&self) -> Vec<SessionSummary> {
        let get_activities = self.get("get_activity").expect("Failed to get activity");
        let activity: tautulli::Activity = get_activities.into();
        let session_summaries: Vec<SessionSummary> = activity.sessions.iter().map(|session| {
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
                }
            }
        }).collect();
        session_summaries
    }
}
