pub mod jellyfin;
pub mod overseerr;
pub mod plex;
pub mod radarr;
//pub mod readarr;
pub mod sonarr;
pub mod structs;
pub mod tautulli;
pub mod unifi;

#[derive(Debug)]
pub enum ProviderErrorKind {
    GetError,
    HeaderError,
    ParseError,
}

#[derive(Debug)]
pub enum Provider {
    Radarr,
    Sonarr,
    Overseerr,
    Tautulli,
    //Unifi,
    //Readarr,
    Reqwest,
    Plex,
    Jellyfin,
}
impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Radarr => write!(f, "Radarr"),
            //Provider::Readarr => write!(f, "Readarr"),
            Provider::Sonarr => write!(f, "Sonarr"),
            Provider::Overseerr => write!(f, "Overseerr"),
            Provider::Tautulli => write!(f, "Tautulli"),
            Provider::Plex => write!(f, "Plex"),
            Provider::Jellyfin => write!(f, "Jellyfin"),
            //Provider::Unifi => write!(f, "Unifi"),
            Provider::Reqwest => write!(f, "Reqwest"),
        }
    }
}

#[derive(Debug)]
pub struct ProviderError {
    provider: Provider,
    kind: ProviderErrorKind,
    message: String,
}
impl ProviderError {
    pub fn new(provider: Provider, kind: ProviderErrorKind, message: &str) -> ProviderError {
        ProviderError {
            provider,
            kind,
            message: message.to_string(),
        }
    }
}
impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ProviderErrorKind::GetError => write!(
                f,
                "There was an error while getting information from {}: {}",
                self.provider, self.message
            ),
            ProviderErrorKind::HeaderError => {
                write!(
                    f,
                    "There was an error while setting headers for {}: {}",
                    self.provider, self.message
                )
            }
            ProviderErrorKind::ParseError => {
                write!(
                    f,
                    "There was an error while parsing {} data: {}",
                    self.provider, self.message
                )
            }
        }
    }
}
impl std::error::Error for ProviderError {}
impl From<reqwest::Error> for ProviderError {
    fn from(e: reqwest::Error) -> ProviderError {
        ProviderError::new(
            Provider::Reqwest,
            ProviderErrorKind::GetError,
            &format!("{:?}", e),
        )
    }
}
