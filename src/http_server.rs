use rocket::{get, routes, Build, Responder, Rocket, State};
use rocket::http::{Accept, ContentType, Header, MediaType, QMediaType, Status};
use rocket::tokio::task;
use rocket::tokio::task::JoinSet;
use once_cell::sync::Lazy;
use tokio::task::JoinError;
use log::{error, info,trace};
use std::process::exit;
use std::cmp::Ordering;


use crate::prometheus::{format_sonarr_metrics, format_tautulli_metrics, Format, TaskResult, format_metrics};
use crate::config::{Config, get_tasks, Task};
use crate::providers::sonarr::{SonarrEpisode, Sonarr};
use crate::providers::tautulli::{Tautulli, SessionSummary};

#[derive(Responder, Debug, PartialEq, Eq)]
#[response(content_type = "text/plain; charset=utf-8")]
pub struct MetricsError {
    response: (Status, String),
}
impl MetricsError {
    fn new(status: Status, response: String) -> Self {
        Self {
            response: (status, response),
        }
    }
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
        .unwrap_or_else(exit_if_handle_fatal);
    rocket::custom(config.http)
        .manage(config.sonarr)
        .manage(tasks)
        .mount("/", routes![index,metrics])
}

#[get("/")]
#[allow(clippy::needless_pass_by_value)]
fn index(
) -> Result<String,MetricsError> {
    let mut response = "Hello Homers".to_string(); 
    Ok(response)
}


#[get("/metrics")]
async fn metrics(
    unscheduled_tasks: &State<Vec<Task>>,
    accept: &Accept,
) -> Result<MetricsResponse,MetricsError> {
    Ok(serve_metrics(get_metrics_format(accept), unscheduled_tasks).await)
}

async fn serve_metrics(
    format: Format,
    unscheduled_tasks: &State<Vec<Task>>,
) -> MetricsResponse {
    let mut join_set = JoinSet::new();

    for task in unscheduled_tasks.iter().cloned() {
        join_set.spawn(task::spawn_blocking(move || {
            info!(
                "Requesting data for {:?}",
                &task,
            );
            match task {
                Task::Sonarr(sonarr) => {
                    let sonarr = Sonarr::new(sonarr.address, sonarr.api_key);
                    let result = sonarr.get_today_shows();
                    TaskResult::Sonarr(result)
                },
                Task::Tautulli(tautulli) => {
                    let tautulli = Tautulli::new(tautulli.address, tautulli.api_key);
                    let result = tautulli.get_session_summary();
                    TaskResult::Tautulli(result)
                },
                Task::Default => TaskResult::Default,
            }
        }));
    }

    wait_for_metrics(format,join_set).await.map_or_else(
        |e| {
            error!("General error while fetching helm release data: {e}");
            MetricsResponse::new(
                Status::InternalServerError,
                format,
                "Error while fetching vault data. Check the logs".into(),
            )
        },
        |metrics| MetricsResponse::new(Status::Ok, format, metrics),
    )
}

async fn wait_for_metrics(
    _format: Format,
    mut join_set: JoinSet<Result<TaskResult, JoinError>>,
) -> anyhow::Result<String> {
    //let mut sonarr_metrics = Vec::new(); 
    //let mut tautulli_metrics = Vec::new();
    let mut metrics = String::new();
    while let Some(result) = join_set.join_next().await {
        match result? {
            Ok(tr) => {
                let formated_metrics = format_metrics(tr);
                metrics.push_str(&formated_metrics?);
            }
            Err(e) => {
                error!("Error while fetching metrics: {e}");
            }
        }
    }
    Ok(metrics)
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

fn sort_media_types_by_priority(accept: &Accept) -> Vec<&QMediaType> {
    let mut vec: Vec<&QMediaType> = accept.iter().collect();
    vec.sort_by(|&left, &right| {
        right
            .weight()
            .map_or(Ordering::Greater, |right_weight| {
                // Absence of weight parameter means most important
                left.weight().map_or(Ordering::Less, |left_weight| {
                    // The higher the weight, the higher the priority
                    right_weight
                        .partial_cmp(&left_weight)
                        .unwrap_or(Ordering::Equal)
                })
            })
            // The more specific, the higher the priority
            .then_with(|| right.specificity().cmp(&left.specificity()))
            // The more parameters, the higher the priority
            .then_with(|| right.params().count().cmp(&left.params().count()))
    });

    trace!("Sorted list of accepted media types: {:#?}", vec);

    vec
}

static OPENMETRICS_CONTENT_TYPE: Lazy<ContentType> = Lazy::new(|| {
    ContentType::new("application", "openmetrics-text")
        .with_params(get_content_type_params("1.0.0"))
});

static TEXT_PLAIN_CONTENT_TYPE: Lazy<ContentType> =
    Lazy::new(|| ContentType::new("text", "plain").with_params(get_content_type_params("0.0.4")));

static MEDIA_TYPE_FORMATS: Lazy<Vec<(&MediaType, Format)>> = Lazy::new(|| {
    vec![
        (OPENMETRICS_CONTENT_TYPE.media_type(), Format::OpenMetrics),
        (TEXT_PLAIN_CONTENT_TYPE.media_type(), Format::Prometheus),
    ]
});

fn get_metrics_format(accept: &Accept) -> Format {
    let media_types_by_priority = sort_media_types_by_priority(accept);

    media_types_by_priority
        .iter()
        .find_map(|&given_media_type| {
            MEDIA_TYPE_FORMATS
                .iter()
                .find_map(|(expected_media_type, format)| {
                    media_type_matches(expected_media_type, given_media_type.media_type())
                        .then_some(*format)
                })
        })
        .unwrap_or(Format::Prometheus)
}
fn media_type_matches(left: &MediaType, right: &MediaType) -> bool {
    left == right || (left.top() == right.top() && (left.sub() == "*" || right.sub() == "*"))
}
