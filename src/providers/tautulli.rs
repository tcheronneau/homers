use reqwest;
use log::debug;
use serde::{Serialize, Deserialize};

use crate::providers::structs::tautulli;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Tautulli {
    pub address: String,
    pub api_key: String,
    api_url: Option<String>,
    #[serde(skip)]
    client: Option<reqwest::blocking::Client>,
}

#[derive(Debug)]
pub struct ActivitySummary {
    stream_count: String,
    sessions: Vec<tautulli::Session>,
}

#[derive(Debug)]
pub struct SessionSummary {
    pub user: String,
    pub title: String,
    pub state: String,
    pub progress: String,
    pub media_type: String,
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
}
impl std::fmt::Display for SessionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.media_type == "episode" {
            write!(f, "User {} is watching {} S{:02}E{:02}. Currently the play is {} and {}% is watched", self.user, self.title, self.season_number.as_ref().unwrap(), self.episode_number.as_ref().unwrap(), self.state, self.progress)
        } else {
            write!(f, "User {} is watching {}. Currently the play is {} and {}% is watched", self.user, self.title, self.state, self.progress)
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
    pub fn get(&self, command: &str) -> anyhow::Result<tautulli::Activity>{
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
    pub fn get_activity_summary(&self) -> anyhow::Result<ActivitySummary> {
        let activity: tautulli::Activity = self.get("get_activity")?;
        Ok(ActivitySummary {
            stream_count: activity.stream_count,
            sessions: activity.sessions,
        })
    }
    pub fn get_session_summary(&self) -> Vec<SessionSummary> {
        let activity: tautulli::Activity = self.get("get_activity").expect("Failed to get activity summary");
        let session_summaries: Vec<SessionSummary> = activity.sessions.iter().map(|session| {
            if session.media_type == "episode" {
                SessionSummary {
                    user: session.user.clone(),
                    title: session.grandparent_title.clone(),
                    state: session.state.clone(),
                    progress: session.progress_percent.clone(),
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
                    media_type: session.media_type.clone(),
                    season_number: None,
                    episode_number: None,
                }
            }
        }).collect();
        session_summaries
    }
}
