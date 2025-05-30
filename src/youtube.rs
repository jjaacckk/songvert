use crate::error::{Error, Result};
use crate::service::{Album, Artist};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct YouTube {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub duration_ms: usize,
    pub music_video: Option<String>,
    pub thumbnails: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawStreamingData {
    expires_in_seconds: String,
    formats: Vec<RawFormat>,
    adaptive_formats: Vec<RawAdaptiveFormat>,
    server_abr_streaming_url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawFormat {
    itag: usize,
    mime_type: String,
    bitrate: usize,
    // pub width: usize,
    // pub height: usize,
    last_modified: String,
    quality: String,
    fps: Option<usize>,
    quality_label: Option<String>,
    projection_type: String,
    audio_quality: String,
    approx_duration_ms: String,
    audio_sample_rate: String,
    audio_channels: usize,
    signature_cipher: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawAdaptiveFormat {
    itag: usize,
    mime_type: String,
    bitrate: usize,
    // pub width: usize,
    // pub height: usize,
    init_range: RawRange,
    index_range: RawRange,
    last_modified: String,
    content_length: String,
    quality: String,
    fps: Option<usize>,
    quality_label: Option<String>,
    projection_type: String,
    average_bitrate: usize,
    approx_duration_ms: String,
    signature_cipher: String,
    // pub color_info: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawRange {
    start: String,
    end: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicShelfRenderer {
    title: RawTextRuns,
    contents: Vec<RawMusicShelfRendererContent>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicShelfRendererContent {
    music_responsive_list_item_renderer: RawMusicResponsiveListItemRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicResponsiveListItemRenderer {
    thumbnail: RawOuterThumbnail,
    flex_columns: Vec<RawFlexColumn>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawFlexColumn {
    music_responsive_list_item_flex_column_renderer: RawMusicResponsiveListItemFlexColumnRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicResponsiveListItemFlexColumnRenderer {
    text: RawTextRuns,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicCardShelfRenderer {
    thumbnail: RawOuterThumbnail,
    title: RawTextRuns,
    subtitle: RawTextRuns,
    header: RawHeader,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawOuterThumbnail {
    music_thumbnail_renderer: RawMusicThumbnailRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicThumbnailRenderer {
    thumbnail: RawInnerThumbnail,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawInnerThumbnail {
    thumbnails: Vec<RawThumbnail>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawThumbnail {
    url: String,
    width: usize,
    height: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawHeader {
    music_card_shelf_header_basic_renderer: RawMusicCardShelfHeaderBasicRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawMusicCardShelfHeaderBasicRenderer {
    title: RawTextRuns,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawTextRuns {
    runs: Vec<RawRun>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawRun {
    text: String,
    navigation_endpoint: Option<RawNavigationEndpoint>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawNavigationEndpoint {
    watch_endpoint: Option<RawWatchEndpoint>,
    browse_endpoint: Option<RawBrowseEndpoint>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawWatchEndpoint {
    video_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawBrowseEndpoint {
    browse_id: String,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Payload<'a> {
    pub context: PayloadContext<'a>,
    pub video_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub params: Option<&'a str>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct PayloadContext<'a> {
    pub client: PayloadContextClient<'a>,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PayloadContextClient<'a> {
    pub hl: &'a str,
    pub gl: &'a str,
    pub client_name: &'a str,
    pub client_version: &'a str,
}

impl YouTube {
    pub const API_BASE_URL: &'static str = "https://www.youtube.com/youtubei/v1";
    pub const SITE_BASE_URL: &'static str = "https://www.youtube.com";
    pub const DEFAULT_MUSIC_PAYLOAD_CONTEXT: PayloadContext<'static> = PayloadContext {
        client: PayloadContextClient {
            hl: "en",
            gl: "US",
            client_name: "WEB_REMIX",
            client_version: "1.20220918",
        },
    };

    async fn post<'a>(client: &Client, path: &str, body: &Payload<'a>) -> Result<Value> {
        let raw_payload: String = serde_json::to_string(&body)?;

        let request: RequestBuilder = client
            .post(format!("{}/{}", Self::API_BASE_URL, path))
            .body(raw_payload);

        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    async fn get_raw_results_from_search(client: &Client, query: &str) -> Result<Value> {
        let payload: Payload = Payload {
            context: Self::DEFAULT_MUSIC_PAYLOAD_CONTEXT,
            video_id: None,
            query: Some(query),
            //params: Some("EgWKAQIIAWoGEAMQChAQ"), // old
            params: Some("EgWKAQIIAUICCAFqBBADEAk"), // this should (among other things) keep API from autocorrecting
        };
        Self::post(client, "search", &payload).await
    }

    async fn get_raw_track_from_id(client: &Client, id: &str) -> Result<Value> {
        let payload: Payload = Payload {
            context: Self::DEFAULT_MUSIC_PAYLOAD_CONTEXT,
            video_id: Some(id),
            query: None,
            params: Some("EgWKAQIIAWoGEAMQChAQ"),
        };
        Self::post(client, "next", &payload).await
    }

    pub async fn download(&self, path: &Path, filename: &str) -> Result<PathBuf> {
        let mut full_path: PathBuf = path.to_owned();
        full_path.push(filename);
        full_path.set_extension("m4a");

        match Command::new("yt-dlp")
            .arg(&self.url)
            .arg("-o")
            .arg(&full_path)
            .arg("-f")
            .arg("m4a")
            .stdout(Stdio::null())
            .status()
        {
            Ok(status) => {
                if status.success() == false {
                    Err(Error::DownloadError("download failed".to_string()))
                } else {
                    Ok(full_path)
                }
            }
            Err(e) => Err(Error::DownloadError(format!("{}", e))),
        }
    }

    async fn get_raw_track_match_from_track(client: &Client, track: &Track) -> Result<Value> {
        let query: String = format!(
            "{}, {}, {}, {}",
            track.name,
            track.artists.get(0).ok_or(Error::TrackError(
                "Track requires at least one artist".to_string(),
            ))?,
            track.release_year,
            track.album,
        );

        Ok(
            Self::get_raw_results_from_search(client, &query).await?["contents"]
                ["tabbedSearchResultsRenderer"]["tabs"]
                .get_mut(0)
                .ok_or(Error::TrackError("no match found".to_string()))?["tabRenderer"]["content"]
                ["sectionListRenderer"]["contents"]
                .take(),
        )
    }

    pub async fn create_service_for_track(client: &Client, track: &mut Track) -> Result<()> {
        let data: Value = Self::get_raw_track_match_from_track(client, track).await?;
        let service: Self = Self::create_service_from_raw(&data, track).await?;
        track.services.youtube = Some(service);
        Ok(())
    }

    async fn create_service_from_raw(data: &Value, track: &Track) -> Result<Self> {
        const TRACK_COMPARE_THRESHOLD: f64 = 2.9;

        let contents: &Vec<Value> = data
            .as_array()
            .ok_or(Error::DatabaseError("no data array".to_string()))?;

        let mut songs: Option<RawMusicShelfRenderer> = None;

        for content in contents {
            if let Ok(music_shelf_renderer) = serde_json::from_value::<RawMusicShelfRenderer>(
                content["musicShelfRenderer"].to_owned(),
            ) {
                if let Some(run) = music_shelf_renderer.title.runs.get(0) {
                    if run.text == "Songs" {
                        songs = Some(music_shelf_renderer);
                        break;
                    }
                }
            }
        }

        let id: &str;
        let mut thumbnails: Vec<String> = Vec::new();

        let songs: RawMusicShelfRenderer =
            songs.ok_or(Error::DatabaseError("no songs".to_string()))?;
        for song in songs.contents {
            let first_flex_run: &Vec<RawRun> = &song
                .music_responsive_list_item_renderer
                .flex_columns
                .get(0)
                .ok_or(Error::DatabaseError("no first flex run".to_string()))?
                .music_responsive_list_item_flex_column_renderer
                .text
                .runs;

            let second_flex_run: &Vec<RawRun> = &song
                .music_responsive_list_item_renderer
                .flex_columns
                .get(1)
                .ok_or(Error::DatabaseError("no second flex run".to_string()))?
                .music_responsive_list_item_flex_column_renderer
                .text
                .runs;

            if first_flex_run.len() < 1 || second_flex_run.len() < 5 {
                return Err(Error::DatabaseError("too few flex runs".to_string()));
            }

            if track.compare_similarity_fuzzy(
                &first_flex_run[0].text,
                &second_flex_run[0].text,
                &second_flex_run[2].text,
                raw_duration_to_miliseconds(&second_flex_run[4].text)?,
            ) >= TRACK_COMPARE_THRESHOLD
            {
                id = &first_flex_run[0]
                    .navigation_endpoint
                    .as_ref()
                    .ok_or(Error::DatabaseError(
                        "no first flex navigation endpoint".to_string(),
                    ))?
                    .watch_endpoint
                    .as_ref()
                    .ok_or(Error::DatabaseError(
                        "no first flex watch endpoint".to_string(),
                    ))?
                    .video_id;

                for thumbnail in song
                    .music_responsive_list_item_renderer
                    .thumbnail
                    .music_thumbnail_renderer
                    .thumbnail
                    .thumbnails
                {
                    thumbnails.push(thumbnail.url.to_owned())
                }

                let artist_id: Option<String> = match &second_flex_run[0].navigation_endpoint {
                    Some(nav_opt) => match &nav_opt.browse_endpoint {
                        Some(nav) => Some(nav.browse_id.to_owned()),
                        None => None,
                    },
                    None => None,
                };

                let album_id: Option<String> = match &second_flex_run[2].navigation_endpoint {
                    Some(nav_opt) => match &nav_opt.browse_endpoint {
                        Some(nav) => Some(nav.browse_id.to_owned()),
                        None => None,
                    },
                    None => None,
                };

                let duration_raw: &str = &second_flex_run[4].text;

                return Ok(Self {
                    id: id.to_owned(),
                    name: first_flex_run[0].text.to_owned(),
                    url: format!("https://www.youtube.com/watch?v={}", id),
                    artists: match artist_id {
                        Some(a_id) => vec![Artist {
                            id: a_id.to_owned(),
                            url: format!("https://music.youtube.com/browse/{}", a_id),
                            name: second_flex_run[0].text.to_owned(),
                        }],
                        None => vec![],
                    },
                    album: match album_id {
                        Some(a_id) => Some(Album {
                            id: a_id.to_owned(),
                            name: second_flex_run[2].text.to_owned(),
                            url: format!("https://music.youtube.com/browse/{}", a_id),
                            total_tracks: None,
                            ean: None,
                            upc: None,
                        }),
                        None => None,
                    },
                    duration_ms: raw_duration_to_miliseconds(duration_raw)?,
                    music_video: None,
                    thumbnails,
                });
            }
        }
        Err(Error::TrackError("unable to create Track".to_string()))
    }
}

fn raw_duration_to_miliseconds(raw_duration: &str) -> Result<usize> {
    let raw_parts = raw_duration.split(':');
    let mut base: usize = 1;
    let mut seconds: usize = 0;
    for part in raw_parts.rev() {
        let num: usize = match part.parse() {
            Ok(n) => n,
            Err(..) => 0,
        };

        seconds += num * base;
        base *= 60;
    }

    Ok(seconds * 1000)
}

#[cfg(test)]
mod tests {

    use crate::{
        service::{Services, Source},
        track::Track,
    };

    #[tokio::test]
    async fn get_match() {
        let example_services: Services = Services {
            spotify: None,
            apple_music: None,
            youtube: None,
            bandcamp: None,
        };

        let mut example_track: Track = Track {
            name: "Duchess for Nothing".to_owned(),
            album: "Genius Fatigue".to_owned(),
            disk_number: 1,
            track_number: 1,
            artists: vec!["Tunabunny".to_owned()],
            release_year: 2013,
            release_month: None,
            release_day: None,
            is_explicit: false,
            duration_ms: 138026,
            services: example_services,
            isrc: Some("USZUD1215001".to_owned()),
            source_service: Source::Spotify,
        };

        let client: reqwest::Client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                    .build()
                    .unwrap();

        example_track.add_youtube(&client).await.unwrap();
    }
}
