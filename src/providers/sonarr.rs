use chrono::{format::strftime::StrftimeItems, Duration, Local};
use log::{debug, error};
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::providers::structs::sonarr;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Sonarr {
    pub address: String,
    #[serde(rename = "apikey")]
    pub api_key: String,
    #[serde(skip)]
    client: reqwest::Client,
}

#[derive(Debug)]
enum SonarrErrorKind {
    GetError,
    ParseError,
}
#[derive(Debug)]
struct SonarrError {
    kind: SonarrErrorKind,
    message: String,
}
impl SonarrError {
    pub fn new(kind: SonarrErrorKind, message: &str) -> SonarrError {
        SonarrError {
            kind,
            message: message.to_string(),
        }
    }
}
impl std::fmt::Display for SonarrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SonarrErrorKind::GetError => write!(
                f,
                "There was an error while getting information from sonarr : {}",
                self.message
            ),
            SonarrErrorKind::ParseError => {
                write!(
                    f,
                    "There was an error while parsing sonarr data: {}",
                    self.message
                )
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SonarrEpisode {
    pub sxe: String,
    pub season_number: i64,
    pub episode_number: i64,
    pub title: String,
    pub serie: String,
    pub air_date: String,
    #[serde(rename = "hasFile")]
    pub has_file: bool,
}
impl std::fmt::Display for SonarrEpisode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {} - {} - {}",
            self.serie, self.sxe, self.title, self.air_date, self.has_file
        )
    }
}

impl Sonarr {
    pub fn new(address: &str, api_key: &str) -> anyhow::Result<Sonarr> {
        let mut headers = header::HeaderMap::new();
        let mut header_api_key = header::HeaderValue::from_str(api_key).unwrap();
        header_api_key.set_sensitive(true);
        headers.insert("X-Api-Key", header_api_key);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Sonarr {
            address: address.to_string(),
            api_key: api_key.to_string(),
            client,
        })
    }
    async fn get_last_seven_days_calendars(&self) -> Result<Vec<sonarr::Calendar>, SonarrError> {
        let url = format!("{}/api/v3/calendar", self.address);
        let local_datetime = Local::now();
        let date_end = local_datetime.date_naive();
        let date_start = date_end - Duration::days(7);
        let format = StrftimeItems::new("%Y-%m-%d");
        let start_date = date_start.format_with_items(format.clone()).to_string();
        let end_date = date_end.format_with_items(format).to_string();

        let params = [
            ("start", &start_date),
            ("end", &end_date),
            ("includeSeries", &true.to_string()),
        ];
        debug!("Params: {:?}", params);
        let response = match self.client.get(&url).query(&params).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(SonarrError::new(
                    SonarrErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let calendars = match response.json::<Vec<sonarr::Calendar>>().await {
            Ok(calendars) => calendars,
            Err(e) => {
                return Err(SonarrError::new(
                    SonarrErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(calendars)
    }
    async fn get_today_calendars(&self) -> Result<Vec<sonarr::Calendar>, SonarrError> {
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
        let params = [
            ("start", &formatted_date_start),
            ("end", &formatted_date_end),
            ("includeSeries", &true.to_string()),
        ];
        let response = match self.client.get(url).query(&params).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(SonarrError::new(
                    SonarrErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let calendars = match response.json::<Vec<sonarr::Calendar>>().await {
            Ok(calendars) => calendars,
            Err(e) => {
                return Err(SonarrError::new(
                    SonarrErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(calendars)
    }

    pub async fn get_today_shows(&self) -> Vec<SonarrEpisode> {
        let calendars = match self.get_today_calendars().await {
            Ok(calendars) => calendars,
            Err(e) => {
                error!("Failed to get today's shows: {:?}", e);
                return Vec::new();
            }
        };
        calendars
            .into_iter()
            .map(|calendar| {
                debug!("{:?}", calendar);
                SonarrEpisode {
                    sxe: format!(
                        "S{:02}E{:02}",
                        calendar.season_number, calendar.episode_number
                    ),
                    season_number: calendar.season_number,
                    episode_number: calendar.episode_number,
                    title: calendar.title.clone(),
                    serie: calendar.series.title.clone(),
                    air_date: calendar.air_date.clone(),
                    has_file: calendar.has_file,
                }
            })
            .collect()
    }

    pub async fn get_last_week_missing_shows(&self) -> Vec<SonarrEpisode> {
        let calendars = match self.get_last_seven_days_calendars().await {
            Ok(calendars) => calendars,
            Err(e) => {
                error!("Failed to get today's shows: {:?}", e);
                return Vec::new();
            }
        };
        calendars
            .iter()
            .filter_map(|calendar| {
                if !calendar.has_file {
                    Some(SonarrEpisode {
                        sxe: format!(
                            "S{:02}E{:02}",
                            calendar.season_number, calendar.episode_number
                        ),
                        season_number: calendar.season_number,
                        episode_number: calendar.episode_number,
                        title: calendar.title.clone(),
                        serie: calendar.series.title.clone(),
                        air_date: calendar.air_date.clone(),
                        has_file: calendar.has_file,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    async fn _get_status(&self) -> sonarr::Status {
        let url = format!("{}/api/v3/system/status", self.address);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .expect("Failed to get sonarr status");
        response.json().await.unwrap()
    }
    async fn _debug(&self, uri: &str) -> String {
        let url = format!("{}/api/v3/{}", self.address, uri);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .expect("Failed to get sonarr status");
        response.text().await.unwrap()
    }
}
