use anyhow::Result;
use axum::{
    extract::State,
    http::header::CONTENT_TYPE,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::extract::TypedHeader;
use futures::future::try_join_all;
use headers::HeaderMap;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task;
use tokio::task::JoinError;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::config::{get_tasks, Config};
use crate::prometheus::{format_metrics, Format};
use crate::tasks::{
    LibraryResult, OverseerrRequestResult, RadarrMovieResult, SessionResult, SonarrEpisodeResult,
    SonarrMissingResult, Task, TaskResult, TautulliLibraryResult, TautulliSessionResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub address: String,
    pub port: u16,
}
impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            address: "localhost".to_string(),
            port: 8000,
        }
    }
}

#[derive(Debug)]
pub struct MetricsError {
    pub message: String,
}

impl IntoResponse for MetricsError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.message).into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    tasks: Vec<Task>,
}

pub async fn configure_axum(config: Config) -> Result<(), anyhow::Error> {
    let config_clone = config.clone();
    let tasks = task::spawn_blocking(move || get_tasks(config_clone))
        .await
        .unwrap_or_else(exit_if_handle_fatal)
        .unwrap_or_else(exit_if_handle_fatal);

    let shared_state = Arc::new(AppState { tasks });
    let app = Router::new()
        .route("/", get(index))
        .route("/metrics", get(metrics))
        .with_state(shared_state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let http_config = config.http.unwrap_or_default();
    let listen = format!("{}:{}", &http_config.address, &http_config.port);
    let listener = tokio::net::TcpListener::bind(&listen).await.unwrap();
    tracing::info!("Starting server on {}", listen);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn index() -> impl IntoResponse {
    Html("Hello Homers".to_string())
}

async fn metrics(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, MetricsError> {
    let mut format = Format::Prometheus;
    if let Some(accept) = headers.get("accept") {
        if let Ok(accept_value) = accept.to_str() {
            if accept_value.contains("openmetrics") {
                format = Format::OpenMetrics;
            };
        };
    };
    dbg!(&format);
    Ok(serve_metrics(format, app_state.tasks.clone()).await)
}

async fn process_tasks(tasks: Vec<Task>) -> Result<Vec<TaskResult>, JoinError> {
    let task_futures: Vec<_> = tasks
        .into_iter()
        .map(|task| async {
            info!("Requesting data for {:?}", &task,);
            match task {
                Task::SonarrToday(sonarr) => {
                    let name = &sonarr.name;
                    let result = sonarr.get_today_shows().await;
                    let result = SonarrEpisodeResult {
                        name: name.to_string(),
                        episodes: result,
                    };
                    Ok(TaskResult::SonarrToday(result))
                }
                Task::SonarrMissing(sonarr) => {
                    let name = &sonarr.name;
                    let result = sonarr.get_last_week_missing_shows().await;
                    let result = SonarrMissingResult {
                        name: name.to_string(),
                        episodes: result,
                    };
                    Ok(TaskResult::SonarrMissing(result))
                }
                Task::TautulliSession(tautulli) => {
                    let result = tautulli.get_session_summary().await;
                    let result = TautulliSessionResult { sessions: result };
                    Ok(TaskResult::TautulliSession(result))
                }
                Task::TautulliLibrary(tautulli) => {
                    let result = tautulli.get_libraries().await;
                    let result = TautulliLibraryResult { libraries: result };
                    Ok(TaskResult::TautulliLibrary(result))
                }
                Task::Radarr(radarr) => {
                    let name = &radarr.name;
                    let result = radarr.get_radarr_movies().await;
                    let result = RadarrMovieResult {
                        name: name.to_string(),
                        movies: result,
                    };
                    Ok(TaskResult::Radarr(result))
                }
                Task::Overseerr(overseerr) => {
                    let result = overseerr.get_overseerr_requests().await;
                    let result = OverseerrRequestResult {
                        kind: "overseerr".to_string(),
                        requests: result,
                    };
                    Ok(TaskResult::Overseerr(result))
                }
                Task::Jellyseerr(overseerr) => {
                    let result = overseerr.get_overseerr_requests().await;
                    let result = OverseerrRequestResult {
                        kind: "jellyseerr".to_string(),
                        requests: result,
                    };
                    Ok(TaskResult::Jellyseerr(result))
                }
                Task::PlexSession(plex) => {
                    let name = &plex.name;
                    let result = plex.get_current_sessions().await;
                    let users = plex.get_users().await;
                    let result = SessionResult {
                        name: name.to_string(),
                        kind: "plex".to_string(),
                        users,
                        sessions: result,
                    };
                    Ok(TaskResult::PlexSession(result))
                }
                Task::PlexLibrary(plex) => {
                    let name = &plex.name;
                    let result = plex.get_all_library_size().await;
                    let result = LibraryResult {
                        name: name.to_string(),
                        kind: "plex".to_string(),
                        libraries: result,
                    };
                    Ok(TaskResult::PlexLibrary(result))
                }
                Task::JellyfinSession(jellyfin) => {
                    let name = &jellyfin.name;
                    let result = jellyfin.get_current_sessions().await;
                    let users = jellyfin.get_users().await;
                    let result = SessionResult {
                        name: name.to_string(),
                        kind: "jellyfin".to_string(),
                        users,
                        sessions: result,
                    };
                    Ok(TaskResult::JellyfinSession(result))
                }
                Task::JellyfinLibrary(jellyfin) => {
                    let name = &jellyfin.name;
                    let result = jellyfin.get_library().await;
                    let result = LibraryResult {
                        name: name.to_string(),
                        kind: "jellyfin".to_string(),
                        libraries: result,
                    };
                    Ok(TaskResult::JellyfinLibrary(result))
                }
                Task::Default => Ok(TaskResult::Default),
            }
        })
        .collect();
    try_join_all(task_futures).await
}

async fn serve_metrics(format: Format, tasks: Vec<Task>) -> impl IntoResponse {
    let content_type = match format {
        Format::OpenMetrics => get_openmetrics_content_type(),
        Format::Prometheus => get_text_plain_content_type(),
    };
    match process_tasks(tasks).await {
        Ok(task_results) => match format_metrics(task_results) {
            Ok(metrics) => {
                (StatusCode::OK, [(CONTENT_TYPE, content_type)], metrics).into_response()
            }
            Err(e) => {
                error!("Error formatting metrics: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(CONTENT_TYPE, get_text_plain_content_type())],
                    "Error formatting metrics. Check the logs.".to_string(),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Error while processing tasks: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, get_text_plain_content_type())],
                "Error while fetching provider data. Check the logs.".to_string(),
            )
                .into_response()
        }
    }
}
fn get_openmetrics_content_type() -> &'static str {
    "application/openmetrics-text; version=1.0.0; charset=utf-8"
}

fn get_text_plain_content_type() -> &'static str {
    "text/plain; version=0.0.4; charset=utf-8"
}

pub fn exit_if_handle_fatal<E, R>(error: E) -> R
where
    E: std::fmt::Display,
{
    error!("Fatal error: {error}");
    std::process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::Task;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_metrics() {
        let shared_state = Arc::new(AppState { tasks: vec![] });
        let result = metrics(State(shared_state), HeaderMap::new()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_tasks() {
        let tasks = vec![Task::Default];
        let result = process_tasks(tasks).await;
        assert!(result.is_ok());
    }
}
