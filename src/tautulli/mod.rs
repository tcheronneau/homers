// Purpose: Tautulli API wrapper
use reqwest;
use log::debug;
use serde::Deserialize;

pub mod structs;

#[derive(Debug, Deserialize)]
pub struct Tautulli {
    pub address: String,
    pub api_key: String,
    api_url: Option<String>,
    #[serde(skip)]
    client: Option<reqwest::Client>,
}

#[derive(Debug)]
pub struct ActivitySummary {
    stream_count: String,
    sessions: Vec<structs::Session>,
}

#[derive(Debug)]
pub struct SessionSummary {
    pub user: String,
    pub title: String,
    pub state: String,
    pub progress: String,
}
impl std::fmt::Display for SessionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User {} is watching {}. Currently the play is {} and {}% is watched", self.user, self.title, self.state, self.progress)
    }
}

impl Tautulli {
    pub fn new(address: String, api_key: String) -> Tautulli {
        let api_url = format!("{}/api/v2?apikey={}&cmd=", address, api_key);
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create tautulli client");
        Tautulli {
            api_key,
            address,
            api_url: Some(api_url),
            client: Some(client),
        }
    }
    pub async fn get(&self, command: &str) -> anyhow::Result<structs::Activity>{
        let url = format!("{}{}", self.api_url.as_ref().unwrap(), command);
        let response = self.client
            .as_ref()
            .expect("Failed to get client")
            .get(&url)
            .send()
            .await
            .expect("Failed to send request");
        let response = response.text().await.expect("Failed to get response text");
        debug!("{}", response);
        let tautulli_response: structs::TautulliResponse = serde_json::from_str(&response).expect("Failed to parse JSON");
        Ok(tautulli_response.response.data)
    }
    pub async fn get_activity_summary(&self) -> anyhow::Result<ActivitySummary> {
        let activity: structs::Activity = self.get("get_activity").await?;
        Ok(ActivitySummary {
            stream_count: activity.stream_count,
            sessions: activity.sessions,
        })
    }
    pub async fn get_session_summary(&self) -> anyhow::Result<Vec<SessionSummary>> {
        let activity: structs::Activity = self.get("get_activity").await?;
        let session_summaries: Vec<SessionSummary> = activity.sessions.iter().map(|session| {
            SessionSummary {
                user: session.user.clone(),
                title: session.title.clone(),
                state: session.state.clone(),
                progress: session.progress_percent.clone(),
            }
        }).collect();
        Ok(session_summaries)
    }
}
