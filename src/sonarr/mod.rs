use reqwest::header;
use chrono::{Local, format::strftime::StrftimeItems, Duration};
use serde::Deserialize;

pub mod structs;

#[derive(Debug, Deserialize)]
pub struct Sonarr<'a> {
    pub address: String,
    #[serde(borrow)]
    pub api_key: &'a str,
    #[serde(skip)]
    client: Option<reqwest::Client>,
}

#[derive(Debug)]
pub struct SonarrEpisode {
    pub sxe: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub serie: String,
    pub air_date: String,
    pub grabbed: bool,
}
impl std::fmt::Display for SonarrEpisode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} - {} - {} - {}", self.serie, self.sxe, self.title, self.air_date, self.grabbed)
    }
}

impl Sonarr <'_> {
    pub fn new(address: String, api_key: &'static str) -> Sonarr{
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = header::HeaderValue::from_static(api_key);
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create sonarr client");
        Sonarr {
            address,
            api_key,
            client: Some(client),
        }
    }
    async fn get_today_calendars(&self) -> Vec<structs::Calendar> {
        let url = format!("{}/api/v3/calendar", self.address);
        let local_datetime = Local::now();

        // Extract the date component
        let date_start = local_datetime.date_naive();
        let date_end = date_start + Duration::days(1);

        // Define the format you want (YYYY-MM-DD)
        let format = StrftimeItems::new("%Y-%m-%d");

        // Format the date as a string
        let formatted_date_start = date_start.format_with_items(format.clone()).to_string();
        let formatted_date_end = date_end.format_with_items(format).to_string();
        let params = [("start", &formatted_date_start), ("end", &formatted_date_end), ("includeSeries", &true.to_string())];
        let response = self.client
            .as_ref()
            .expect("Sonarr client not initialized")
            .get(url)
            .query(&params)
            .send()
            .await
            .expect("Failed to get sonarr calendar");
        response.json().await.unwrap()
    }

    pub async fn get_today_shows(&self) -> Vec<SonarrEpisode> {
        let calendars = self.get_today_calendars().await;
        calendars.iter().map(|calendar| {
            SonarrEpisode {
                sxe: format!("S{:02}E{:02}", calendar.season_number, calendar.episode_number),
                season_number: calendar.season_number,
                episode_number: calendar.episode_number,
                title: calendar.title.clone(),
                serie: calendar.series.title.clone(),
                air_date: calendar.air_date.clone(),
                grabbed: calendar.grabbed,
            }
        }).collect()
    }

    pub async fn get_status(&self) -> structs::Status {
        let url = format!("{}/api/v3/system/status", self.address);
        let response = self.client
            .as_ref()
            .expect("Sonarr client not initialized")
            .get(url)
            .send()
            .await
            .expect("Failed to get sonarr status");
        response.json().await.unwrap()
    }
    pub async fn debug(&self, uri: &str) -> String {
        let url = format!("{}/api/v3/{}", self.address, uri);
        let response = self.client
            .as_ref()
            .expect("Sonarr client not initialized")
            .get(url)
            .send()
            .await
            .expect("Failed to get sonarr status");
        response.text().await.unwrap()
    }
}
