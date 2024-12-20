use crate::providers::structs::AsyncFrom;
use log::{debug, error, info};
use reqwest;
use reqwest::header;
use serde::{Deserialize, Serialize};

pub use crate::providers::structs::plex::{LibraryInfos, MediaContainer};
use crate::providers::structs::plex::{Metadata, PlexResponse, StatUser};
use crate::providers::structs::{LibraryCount, Session, User};
use crate::providers::{Provider, ProviderError, ProviderErrorKind};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PlexViews {
    pub episodes_viewed: i64,
    pub movies_viewed: i64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Plex {
    #[serde(skip)]
    pub name: String,
    pub address: String,
    pub token: String,
    #[serde(skip)]
    pub client: reqwest::Client,
}
impl Plex {
    pub fn _default() -> Plex {
        match Plex::new("default", "http://localhost:32400", "123456789") {
            Ok(plex) => plex,
            Err(e) => {
                eprintln!("Failed to create default Plex struct: {}", e);
                std::process::exit(1);
            }
        }
    }
    pub fn new(name: &str, address: &str, token: &str) -> anyhow::Result<Plex> {
        let mut headers = header::HeaderMap::new();
        let mut header_token = header::HeaderValue::from_str(&token)?;
        let header_container_size = header::HeaderValue::from_static("1000");
        header_token.set_sensitive(true);
        headers.insert("X-Plex-Token", header_token);
        headers.insert("X-Plex-Container-Size", header_container_size);
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Plex {
            name: name.to_string(),
            address: address.to_string(),
            token: token.to_string(),
            client,
        })
    }
    async fn _get_history(&self) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/status/sessions/history/all", self.address);
        debug!("Requesting history from {}", url);
        let response = self.client.get(&url).send().await?;
        let history = response.json::<PlexResponse>().await?;
        Ok(history)
    }

    async fn get_sessions(&self) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/status/sessions", self.address);
        debug!("Requesting session from {}", url);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Plex,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let session = match response.json::<PlexResponse>().await {
            Ok(session) => session,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Plex,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(session)
    }
    async fn get_all_libraries(&self) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/library/sections", self.address);
        debug!("Requesting libraries from {}", url);
        let response = self.client.get(&url).send().await?;
        let libraries = response.json::<PlexResponse>().await?;
        Ok(libraries)
    }

    async fn get_library_items(&self, library_id: &str) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/library/sections/{}/all", self.address, library_id);
        debug!("Requesting library items from {}", url);
        let response = self.client.get(&url).send().await?;
        let library_items = response.json::<PlexResponse>().await?;
        Ok(library_items)
    }

    pub async fn get_all_library_size(&self) -> Vec<LibraryCount> {
        let libraries = match self.get_all_libraries().await {
            Ok(libraries) => libraries,
            Err(e) => {
                error!("Failed to get libraries: {}", e);
                return Vec::new();
            }
        };
        let mut library_infos: Vec<LibraryInfos> = Vec::new();
        let libraries_container = match libraries.media_container {
            MediaContainer::LibraryContainer(libraries_container) => libraries_container,
            _ => {
                error!("Media container received does not match library container");
                return Vec::new();
            }
        };
        for item in libraries_container.directory {
            let library_item = match self.get_library_items(&item.key).await {
                Ok(library_item) => library_item,
                Err(e) => {
                    error!("Failed to get library items: {}", e);
                    return Vec::new();
                }
            };
            let library_items_container = match library_item.media_container {
                MediaContainer::LibraryItemsContainer(library_items_container) => {
                    library_items_container
                }
                _ => {
                    error!("Media container received does not match library items container");
                    return Vec::new();
                }
            };
            match &item.type_field[..] {
                "show" => {
                    let (child_sum, leaf_sum) = library_items_container.metadata.iter().fold(
                        (0, 0),
                        |(mut child_acc, mut leaf_acc), child| {
                            match child {
                                Metadata::LibraryMetadata(meta) => {
                                    child_acc += meta.child_count.unwrap_or(0);
                                    leaf_acc += meta.leaf_count.unwrap_or(0);
                                }
                                _ => {
                                    error!("Metadata received does not match library metadata");
                                }
                            }
                            (child_acc, leaf_acc)
                        },
                    );
                    library_infos.push(LibraryInfos {
                        library_name: item.title.to_string(),
                        library_type: item.type_field.to_string(),
                        library_size: library_items_container.size,
                        library_child_size: Some(child_sum),
                        library_grand_child_size: Some(leaf_sum),
                    });
                }
                _ => library_infos.push(LibraryInfos {
                    library_name: item.title.to_string(),
                    library_type: item.type_field.to_string(),
                    library_size: library_items_container.size,
                    library_child_size: None,
                    library_grand_child_size: None,
                }),
            }
        }
        library_infos.into_iter().map(|item| item.into()).collect()
    }

    pub async fn get_current_sessions(&self) -> Vec<Session> {
        let sessions = match self.get_sessions().await {
            Ok(sessions) => sessions,
            Err(e) => {
                error!("Failed to get sessions: {}", e);
                return Vec::new();
            }
        };
        let mut current_sessions: Vec<Session> = Vec::new();
        let activity_container = match sessions.media_container {
            MediaContainer::ActivityContainer(activity_container) => activity_container,
            _ => {
                error!("Media container received does not match activity container");
                return Vec::new();
            }
        };
        for item in activity_container.metadata.into_iter() {
            match item {
                Metadata::SessionMetadata(meta) => {
                    let session = Session::from_async(meta).await;
                    current_sessions.push(session);
                }
                _ => {
                    error!("Metadata received does not match session metadata");
                }
            }
        }
        current_sessions
    }

    pub async fn _get_views(&self) -> PlexViews {
        let history = match self._get_history().await {
            Ok(history) => history,
            Err(e) => {
                error!("Failed to get history: {}", e);
                return PlexViews {
                    episodes_viewed: 0,
                    movies_viewed: 0,
                };
            }
        };
        let mut episodes_viewed = 0;
        let mut movies_viewed = 0;
        let activity_container = match history.media_container {
            MediaContainer::ActivityContainer(activity_container) => activity_container,
            _ => {
                error!("Media container received does not match activity container");
                return PlexViews {
                    episodes_viewed: 0,
                    movies_viewed: 0,
                };
            }
        };
        activity_container
            .metadata
            .iter()
            .for_each(|item| match item {
                Metadata::HistoryMetadata(meta) => {
                    if meta.type_field == "episode" {
                        episodes_viewed += 1;
                    } else if meta.type_field == "movie" {
                        movies_viewed += 1;
                    }
                }
                _ => {
                    error!("Metadata received does not match history metadata");
                }
            });
        PlexViews {
            episodes_viewed,
            movies_viewed,
        }
    }
    pub async fn get_statistics(&self) -> Result<PlexResponse, ProviderError> {
        let url = format!("{}/statistics/bandwidth?timespan=0", self.address);
        debug!("Requesting statistics from {}", url);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Plex,
                    ProviderErrorKind::GetError,
                    &format!("{:?}", e),
                ));
            }
        };
        let statistics = match response.json::<PlexResponse>().await {
            Ok(statistics) => statistics,
            Err(e) => {
                return Err(ProviderError::new(
                    Provider::Plex,
                    ProviderErrorKind::ParseError,
                    &format!("{:?}", e),
                ));
            }
        };
        Ok(statistics)
    }
    pub async fn get_users(&self) -> Vec<User> {
        let statistics = match self.get_statistics().await {
            Ok(statistics) => statistics,
            Err(e) => {
                error!("Failed to get statistics: {}", e);
                return Vec::new();
            }
        };
        let statistics_container = match statistics.media_container {
            MediaContainer::StatisticsContainer(statistics_container) => statistics_container,
            MediaContainer::Default(_) => {
                info!("No session currently");
                return Vec::new();
            }
            _ => {
                error!("Media container received does not match statistics container");
                return Vec::new();
            }
        };
        statistics_container
            .account
            .into_iter()
            .map(|item| <StatUser as Into<User>>::into(item))
            .collect()
    }
}
