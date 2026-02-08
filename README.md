# HOMERS

A Prometheus exporter for home media services, built as a replacement for [Varken](https://github.com/Boerderij/Varken).
Instead of InfluxDB, Homers exposes metrics in Prometheus/OpenMetrics format via an HTTP endpoint at `/metrics`.

![image](https://github.com/user-attachments/assets/9a0c2fb0-52f3-439d-b590-9c6698994d10)

## Supported Services

| Service | Metrics |
|---------|---------|
| **Sonarr** | Today's calendar, missing episodes, episode counts |
| **Radarr** | Movie library (has file, monitored, available), totals |
| **Lidarr** | Artist library (monitored status), track counts |
| **Readarr** | Author library (monitored status), book counts |
| **Tautulli** | Active sessions, library stats, watch history (24h per-user, hourly) |
| **Plex** | Sessions (progress, bandwidth, geo-location), users, library counts |
| **Jellyfin** | Sessions (progress, geo-location), users, library counts |
| **Overseerr** | Request status, media status, request totals by state |
| **Jellyseerr** | Same as Overseerr |

Multi-instance support is available for Sonarr, Radarr, Lidarr, Readarr, Plex, and Jellyfin.

## Getting Started

### Docker

```
docker run -d -p 8000:8000 -v ./config.toml:/app/config.toml mcth/homers
```

Images are available on [Docker Hub](https://hub.docker.com/repository/docker/mcth/homers).

### Configuration

Homers supports both TOML config files and environment variables.

**config.toml:**

```toml
[http]
port = 8000
address = "0.0.0.0"

[sonarr.main]
address = "http://localhost:8989"
apikey = ""

[sonarr.second]
address = "http://localhost:7979"
apikey = ""

[radarr.main]
address = "http://localhost:7878"
apikey = ""

[lidarr.main]
address = "http://localhost:8686"
apikey = ""

[readarr.main]
address = "http://localhost:8787"
apikey = ""

[tautulli]
address = "http://localhost:8181"
apikey = ""

[overseerr]
address = "http://localhost:5055"
apikey = ""
requests = 200

[jellyseerr]
address = "http://localhost:5055"
apikey = ""

[plex.main]
address = "http://localhost:32400"
token = ""

[jellyfin.main]
address = "http://localhost:8096"
apikey = ""
```

**Environment variables:**

Each config key maps to an environment variable with `HOMERS_` prefix and `_` as separator:

```bash
HOMERS_HTTP_ADDRESS="0.0.0.0"
HOMERS_SONARR_MAIN_ADDRESS="http://localhost:8989"
HOMERS_SONARR_MAIN_APIKEY=""
```

For Overseerr/Jellyseerr, the `requests` field controls how many requests to pull (default: 20).

### Multi-instance support

Services like Sonarr, Radarr, Lidarr, Readarr, Plex, and Jellyfin support multiple instances. Use a unique identifier for each instance in the config:

```toml
[sonarr.main]
address = "http://sonarr1:8989"
apikey = ""

[sonarr.anime]
address = "http://sonarr2:8989"
apikey = ""
```

The instance name is exposed as the `name` label in metrics.

## Exposed Metrics

All metrics are prefixed with `homers_`. Below is the full list:

### Sonarr
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `sonarr_today_episode` | gauge | name, series, sxe, has_file | Today's episode info |
| `sonarr_today_episodes_total` | gauge | name | Total episodes airing today |
| `sonarr_missing_episode` | gauge | name, series, sxe, has_file | Missing episode info |
| `sonarr_missing_episodes_total` | gauge | name | Total missing episodes |

### Radarr
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `radarr_movie_has_file` | gauge | name, title | Movie has file on disk |
| `radarr_movie_monitored` | gauge | name, title | Movie is monitored |
| `radarr_movie_available` | gauge | name, title | Movie is available |
| `radarr_movies_total` | gauge | name | Total movie count |
| `radarr_movies_monitored_total` | gauge | name | Monitored movie count |
| `radarr_movies_missing_total` | gauge | name | Missing movie count |

### Lidarr
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `lidarr_artist_monitored` | gauge | name, artist | Artist is monitored |
| `lidarr_artists_total` | gauge | name | Total artist count |
| `lidarr_monitored_artists_total` | gauge | name | Monitored artist count |
| `lidarr_tracks_total` | gauge | name | Total track file count |

### Readarr
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `readarr_author_monitored` | gauge | name, author | Author is monitored |
| `readarr_authors_total` | gauge | name | Total author count |
| `readarr_monitored_authors_total` | gauge | name | Monitored author count |
| `readarr_books_total` | gauge | name | Total book file count |

### Tautulli
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `tautulli_session_count` | gauge | | Current active session count |
| `tautulli_session_info` | gauge | user, title, state, quality, quality_profile, video_stream, media_type, season_number, episode_number | Session info (value=1) |
| `tautulli_session_progress` | gauge | user, title | Session progress percentage |
| `tautulli_session_location` | gauge | user, title, city, country, ip_address, latitude, longitude | Session geo-location info |
| `tautulli_library_item_count` | gauge | section_name, section_type | Library item count |
| `tautulli_library_parent_count` | gauge | section_name, section_type | Library parent count |
| `tautulli_library_child_count` | gauge | section_name, section_type | Library child count |
| `tautulli_library_active` | gauge | section_name, section_type | Library is active |
| `tautulli_history_total_plays` | gauge | | All-time total play count |
| `tautulli_history_user_watches_24h` | gauge | user | Per-user watch count in last 24h |
| `tautulli_history_plays_24h` | gauge | | Total plays in last 24h |

### Plex
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `plex_session_count` | gauge | name | Active session count |
| `plex_session_info` | gauge | name, user, title, state, quality, media_type, season_number, episode_number | Session info |
| `plex_session_progress` | gauge | name, user, title | Session progress percentage |
| `plex_session_bandwidth` | gauge | name, user, title | Session bandwidth |
| `plex_session_location` | gauge | name, user, title, city, country, ip_address, latitude, longitude | Session geo-location |
| `plex_user_active` | gauge | name, user | User is active |
| `plex_library_count` | gauge | name, library | Library item count |
| `plex_library_child_count` | gauge | name, library | Library child count |
| `plex_library_grandchild_count` | gauge | name, library | Library grandchild count |
| `plex_movie_count` | gauge | name | Total movie count |
| `plex_show_count` | gauge | name | Total show count |
| `plex_season_count` | gauge | name | Total season count |
| `plex_episode_count` | gauge | name | Total episode count |

### Jellyfin
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `jellyfin_session_count` | gauge | name | Active session count |
| `jellyfin_session_info` | gauge | name, user, title, state, quality, media_type, season_number, episode_number | Session info |
| `jellyfin_session_progress` | gauge | name, user, title | Session progress percentage |
| `jellyfin_user_active` | gauge | name, user | User is active |
| `jellyfin_session_location` | gauge | name, user, title, city, country, ip_address, latitude, longitude | Session geo-location |
| `jellyfin_library_count` | gauge | name, library | Library item count |
| `jellyfin_library_child_count` | gauge | name, library | Library child count |
| `jellyfin_library_grandchild_count` | gauge | name, library | Library grandchild count |
| `jellyfin_movie_count` | gauge | name | Total movie count |
| `jellyfin_show_count` | gauge | name | Total show count |
| `jellyfin_episode_count` | gauge | name | Total episode count |

### Overseerr / Jellyseerr
| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `{overseerr,jellyseerr}_request_status` | gauge | name, media_type | Request status value |
| `{overseerr,jellyseerr}_media_status` | gauge | name, media_type | Media status value |
| `{overseerr,jellyseerr}_requests_total` | gauge | | Total request count |
| `{overseerr,jellyseerr}_requests_pending_total` | gauge | | Pending request count |
| `{overseerr,jellyseerr}_requests_approved_total` | gauge | | Approved request count |
| `{overseerr,jellyseerr}_requests_declined_total` | gauge | | Declined request count |

## Grafana Dashboard

A pre-built Grafana dashboard is included in `dashboard.json` at the root of this repository. It provides panels for all supported services including session monitoring, library stats, watch history, and request tracking.

A published version is also available at [Grafana Dashboards](https://grafana.com/grafana/dashboards/20744).

## Building

### With Nix (recommended)

```bash
nix build
```

For development:

```bash
nix develop -c cargo build
nix develop -c cargo test
nix develop -c cargo clippy
```

### With Cargo

```bash
cargo build --release
```

### Docker image via Nix

```bash
nix build .#docker
docker load < ./result
```

## Acknowledgments

This project is heavily inspired by the work of [Lars Strojny](https://github.com/lstrojny/prometheus-weathermen), which provides an excellent example of a Prometheus exporter in Rust.

This project is developed with the assistance of AI (Claude Code by Anthropic).
