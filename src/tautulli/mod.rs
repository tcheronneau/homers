// Purpose: Tautulli API wrapper
use reqwest;
use log::debug;

pub mod structs;

pub struct Tautulli {
    server: String,
    api_key: String,
    api_url: Option<String>,
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
impl SessionSummary {
    pub fn to_string(&self) -> String {
        format!("User {} is watching {}. Currently the play is {} and {}% is watched", self.user, self.title, self.state, self.progress)
    }
}

impl Tautulli {
    pub fn new(server: String, api_key: String) -> Tautulli {
        let api_url = format!("{}/api/v2?apikey={}&cmd=", server, api_key);
        Tautulli {
            api_key,
            server,
            api_url: Some(api_url),
        }
    }
    pub fn get(&self, command: &str) -> anyhow::Result<structs::Activity>{
        let url = format!("{}{}", self.api_url.as_ref().unwrap(), command);
        let response = reqwest::blocking::get(&url)
            .expect("Failed to send request")
            .text()
            .expect("Failed to get response body");
        debug!("{}", response);
        let tautulli_response: structs::TautulliResponse = serde_json::from_str(&response).expect("Failed to parse JSON");
        Ok(tautulli_response.response.data)
    }
    pub fn get_activity_summary(&self) -> anyhow::Result<ActivitySummary> {
        let activity: structs::Activity = self.get("get_activity")?;
        Ok(ActivitySummary {
            stream_count: activity.stream_count,
            sessions: activity.sessions,
        })
    }
    pub fn get_session_summary(&self) -> anyhow::Result<Vec<SessionSummary>> {
        let activity: structs::Activity = self.get("get_activity")?;
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
