use anyhow::Result;
use log::{error, info};
use rocket::http::{Accept, ContentType, Status};
use rocket::tokio::task;
use rocket::tokio::task::JoinSet;
use rocket::{get, routes, Build, Responder, Rocket, State};
use std::collections::HashMap;
use std::process::exit;
use tokio::task::JoinError;

use crate::config::{get_tasks, Config, Task};
use crate::prometheus::{format_metrics, Format, TaskResult};

#[derive(Responder, Debug, PartialEq, Eq)]
#[response(content_type = "text/plain; charset=utf-8")]
pub struct MetricsError {
    response: (Status, String),
}

#[derive(Responder, Debug, PartialEq, Eq)]
#[response()]
pub struct MetricsResponse {
    response: (Status, String),
    content_type: ContentType,
}

impl MetricsResponse {
    fn new(status: Status, content_type: Format, response: String) -> Self {
        let content_type = if status.class().is_success() && content_type == Format::OpenMetrics {
            get_openmetrics_content_type()
        } else {
            get_text_plain_content_type()
        };

        Self {
            content_type,
            response: (status, response),
        }
    }
}
pub async fn configure_rocket(config: Config) -> Rocket<Build> {
    let config_clone = config.clone();
    let tasks = task::spawn_blocking(move || get_tasks(config_clone))
        .await
        .unwrap_or_else(exit_if_handle_fatal)
        .unwrap_or_else(exit_if_handle_fatal);
    rocket::custom(config.http)
        .manage(tasks)
        .mount("/", routes![index, metrics])
}

#[get("/")]
#[allow(clippy::needless_pass_by_value)]
fn index() -> Result<String, MetricsError> {
    let response = "Hello Homers".to_string();
    Ok(response)
}

#[get("/metrics")]
async fn metrics(
    unscheduled_tasks: &State<Vec<Task>>,
    _accept: &Accept,
) -> Result<MetricsResponse, MetricsError> {
    Ok(serve_metrics(Format::Prometheus, unscheduled_tasks).await)
}
async fn process_task(task: Task) -> Result<TaskResult, JoinError> {
    info!("Requesting data for {:?}", &task,);
    match task {
        Task::SonarrToday(sonarr) => {
            let name = &sonarr.name;
            let result = sonarr.get_today_shows().await;
            let result = HashMap::from([(name.to_string(), result)]);
            Ok(TaskResult::SonarrToday(result))
        }
        Task::SonarrMissing(sonarr) => {
            let name = &sonarr.name;
            let result = sonarr.get_last_week_missing_shows().await;
            let result = HashMap::from([(name.to_string(), result)]);
            Ok(TaskResult::SonarrMissing(result))
        }
        Task::TautulliSession(tautulli) => {
            let result = tautulli.get_session_summary().await;
            Ok(TaskResult::TautulliSession(result))
        }
        Task::TautulliLibrary(tautulli) => {
            let result = tautulli.get_libraries().await;
            Ok(TaskResult::TautulliLibrary(result))
        }
        Task::Radarr(radarr) => {
            let name = &radarr.name;
            let result = radarr.get_radarr_movies().await;
            let result = HashMap::from([(name.to_string(), result)]);
            Ok(TaskResult::Radarr(result))
        }
        Task::Overseerr(overseerr) => {
            let result = overseerr.get_overseerr_requests().await;
            Ok(TaskResult::Overseerr(result))
        }
        Task::PlexSession(plex) => {
            let name = &plex.name;
            let result = plex.get_current_sessions().await;
            let result = HashMap::from([(name.to_string(), result)]);
            Ok(TaskResult::PlexSession(result))
        }
        Task::PlexLibrary(plex) => {
            let name = &plex.name;
            let result = plex.get_all_library_size().await;
            let result = HashMap::from([(name.to_string(), result)]);
            Ok(TaskResult::PlexLibrary(result))
        }
        Task::Default => Ok(TaskResult::Default),
    }
}

async fn serve_metrics(format: Format, unscheduled_tasks: &State<Vec<Task>>) -> MetricsResponse {
    let mut join_set = JoinSet::new();
    for task in unscheduled_tasks.iter().cloned() {
        join_set.spawn(process_task(task));
    }

    wait_for_metrics(format, join_set).await.map_or_else(
        |e| {
            error!("General error while fetching providers data: {e}");
            MetricsResponse::new(
                Status::InternalServerError,
                format,
                "Error while fetching providers data. Check the logs".into(),
            )
        },
        |metrics| MetricsResponse::new(Status::Ok, format, metrics),
    )
}

async fn wait_for_metrics(
    _format: Format,
    mut join_set: JoinSet<Result<TaskResult, JoinError>>,
) -> anyhow::Result<String> {
    let mut tasks: Vec<TaskResult> = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result? {
            Ok(tr) => {
                tasks.push(tr);
            }
            Err(e) => {
                error!("Error while fetching metrics: {e}");
            }
        }
    }
    format_metrics(tasks)
}

const fn get_content_type_params(version: &str) -> [(&str, &str); 2] {
    [("charset", "utf-8"), ("version", version)]
}

fn get_openmetrics_content_type() -> ContentType {
    ContentType::new("application", "openmetrics-text")
        .with_params(get_content_type_params("1.0.0"))
}

fn get_text_plain_content_type() -> ContentType {
    ContentType::new("text", "plain").with_params(get_content_type_params("0.0.4"))
}

pub fn exit_if_handle_fatal<E, R>(error: E) -> R
where
    E: std::fmt::Display,
{
    error!("Fatal error: {error}");

    exit(1)
}
