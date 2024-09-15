use crate::error::{Error, Result};
use crate::service::{Album, Artist};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct YouTube {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub duration_raw: String,
    pub duration_ms: usize,
    pub music_video: Option<String>,
    pub thumbnails: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawStreamingData {
    pub expires_in_seconds: String,
    pub formats: Vec<RawFormat>,
    pub adaptive_formats: Vec<RawAdaptiveFormat>,
    pub server_abr_streaming_url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawFormat {
    pub itag: usize,
    pub mime_type: String,
    pub bitrate: usize,
    // pub width: usize,
    // pub height: usize,
    pub last_modified: String,
    pub quality: String,
    pub fps: Option<usize>,
    pub quality_label: Option<String>,
    pub projection_type: String,
    pub audio_quality: String,
    pub approx_duration_ms: String,
    pub audio_sample_rate: String,
    pub audio_channels: usize,
    pub signature_cipher: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawAdaptiveFormat {
    pub itag: usize,
    pub mime_type: String,
    pub bitrate: usize,
    // pub width: usize,
    // pub height: usize,
    pub init_range: RawRange,
    pub index_range: RawRange,
    pub last_modified: String,
    pub content_length: String,
    pub quality: String,
    pub fps: Option<usize>,
    pub quality_label: Option<String>,
    pub projection_type: String,
    pub average_bitrate: usize,
    pub approx_duration_ms: String,
    pub signature_cipher: String,
    // pub color_info: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawRange {
    pub start: String,
    pub end: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicShelfRenderer {
    title: RawTextRuns,
    contents: Vec<RawMusicShelfRendererContent>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicShelfRendererContent {
    music_responsive_list_item_renderer: RawMusicResponsiveListItemRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicResponsiveListItemRenderer {
    thumbnail: RawOuterThumbnail,
    flex_columns: Vec<RawFlexColumn>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawFlexColumn {
    music_responsive_list_item_flex_column_renderer: RawMusicResponsiveListItemFlexColumnRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicResponsiveListItemFlexColumnRenderer {
    text: RawTextRuns,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicCardShelfRenderer {
    thumbnail: RawOuterThumbnail,
    title: RawTextRuns,
    subtitle: RawTextRuns,
    header: RawHeader,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawOuterThumbnail {
    music_thumbnail_renderer: RawMusicThumbnailRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicThumbnailRenderer {
    thumbnail: RawInnerThumbnail,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawInnerThumbnail {
    thumbnails: Vec<RawThumbnail>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawThumbnail {
    url: String,
    width: usize,
    height: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawHeader {
    music_card_shelf_header_basic_renderer: RawMusicCardShelfHeaderBasicRenderer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawMusicCardShelfHeaderBasicRenderer {
    title: RawTextRuns,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawTextRuns {
    runs: Vec<RawRun>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawRun {
    text: String,
    navigation_endpoint: Option<RawNavigationEndpoint>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawNavigationEndpoint {
    watch_endpoint: Option<RawWatchEndpoint>,
    browse_endpoint: Option<RawBrowseEndpoint>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawWatchEndpoint {
    video_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawBrowseEndpoint {
    browse_id: String,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Payload<'a> {
    context: PayloadContext<'a>,
    video_id: Option<&'a str>,
    query: Option<&'a str>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct PayloadContext<'a> {
    client: PayloadContextClient<'a>,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PayloadContextClient<'a> {
    hl: &'a str,
    gl: &'a str,
    client_name: &'a str,
    client_version: &'a str,
}

impl YouTube {
    const API_BASE_URL: &'static str = "https://www.youtube.com/youtubei/v1";
    const SITE_BASE_URL: &'static str = "https://www.youtube.com";
    const DEFAULT_MUSIC_PAYLOAD_CONTEXT: PayloadContext<'static> = PayloadContext {
        client: PayloadContextClient {
            hl: "en",
            gl: "US",
            client_name: "WEB_REMIX",
            client_version: "1.20220918",
        },
    };

    pub async fn post<'a>(client: &Client, path: &str, body: &Payload<'a>) -> Result<Value> {
        let raw_payload: String = serde_json::to_string(&body)?;

        let request: RequestBuilder = client
            .post(format!("{}/{}", Self::API_BASE_URL, path))
            .body(raw_payload);

        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    pub async fn get_raw_results_from_search(client: &Client, query: &str) -> Result<Value> {
        // println!("{}", query);

        let payload: Payload = Payload {
            context: Self::DEFAULT_MUSIC_PAYLOAD_CONTEXT,
            video_id: None,
            query: Some(query),
        };
        Self::post(client, "search", &payload).await
    }

    pub async fn get_raw_track_from_id(client: &Client, id: &str) -> Result<Value> {
        let payload: Payload = Payload {
            context: Self::DEFAULT_MUSIC_PAYLOAD_CONTEXT,
            video_id: Some(id),
            query: None,
        };
        Self::post(client, "next", &payload).await
    }

    pub async fn get_raw_track_streaming_data_from_id(
        client: &Client,
        id: &str,
    ) -> Result<RawStreamingData> {
        let request: RequestBuilder = client.get(format!("{}/watch?v={}", Self::SITE_BASE_URL, id));

        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let raw_html: String = response.text().await?;

        let re = regex::Regex::new(r#"var ytInitialPlayerResponse = (.*);</script>"#)?;

        // println!("{}", &raw_html);

        let raw_initial_data: &str = match re.captures(&raw_html) {
            Some(captures) => match captures.get(1) {
                Some(m) => m.as_str(),
                None => return Err(Error::FindError),
            },
            None => return Err(Error::FindError),
        };

        // println!("{}", raw_initial_data);
        // let mut file = std::fs::File::create("./test.js")?;
        // file.write_all(&raw_initial_data.as_str().as_bytes())?;

        let mut initial_data: Value = serde_json::from_str(raw_initial_data)?;

        let streaming_data: RawStreamingData =
            serde_json::from_value(initial_data["streamingData"].take())?;

        Ok(streaming_data)
    }

    pub async fn get_raw_track_match_from_track(client: &Client, track: &Track) -> Result<Value> {
        let query: String = format!(
            "{}, {}, {}, {}",
            track.name,
            track.artists.get(0).ok_or(Error::MalformedTrackError)?,
            track.release_year,
            track.album,
        );

        Ok(
            Self::get_raw_results_from_search(client, &query).await?["contents"]
                ["tabbedSearchResultsRenderer"]["tabs"]
                .get_mut(0)
                .ok_or(Error::MatchError)?["tabRenderer"]["content"]["sectionListRenderer"]
                ["contents"]
                .take(),
        )
    }

    pub async fn create_service_for_track(client: &Client, track: &mut Track) -> Result<()> {
        let data: Value = Self::get_raw_track_match_from_track(client, track).await?;
        let service: Self = Self::create_service_from_raw(&data, track).await?;
        track.services.youtube = Some(service);
        Ok(())
    }

    pub async fn create_service_from_raw(data: &Value, track: &Track) -> Result<Self> {
        let contents: &Vec<Value> = data.as_array().ok_or(Error::CreateError)?;

        let mut top_result: Option<RawMusicCardShelfRenderer> = None;
        let mut songs: Option<RawMusicShelfRenderer> = None;
        let mut videos: Option<RawMusicShelfRenderer> = None;

        for content in contents {
            if top_result == None {
                match serde_json::from_value::<RawMusicCardShelfRenderer>(
                    content["musicCardShelfRenderer"].to_owned(),
                ) {
                    Ok(music_card_shelf_renderer) => {
                        match music_card_shelf_renderer
                            .header
                            .music_card_shelf_header_basic_renderer
                            .title
                            .runs
                            .get(0)
                        {
                            Some(run) => {
                                if run.text == "Top result" {
                                    top_result = Some(music_card_shelf_renderer);
                                    continue;
                                }
                            }
                            None => (),
                        }
                    }
                    Err(..) => (),
                }
            }

            if songs == None || videos == None {
                match serde_json::from_value::<RawMusicShelfRenderer>(
                    content["musicShelfRenderer"].to_owned(),
                ) {
                    Ok(music_shelf_renderer) => match music_shelf_renderer.title.runs.get(0) {
                        Some(run) => {
                            if songs == None && run.text == "Songs" {
                                songs = Some(music_shelf_renderer);
                                continue;
                            } else if videos == None && run.text == "Videos" {
                                videos = Some(music_shelf_renderer);
                                continue;
                            }
                        }
                        None => (),
                    },
                    Err(..) => (),
                }
            };
        }

        let top_result: RawMusicCardShelfRenderer = top_result.ok_or(Error::CreateError)?;

        let top_result_type: &str = &top_result
            .subtitle
            .runs
            .get(0)
            .ok_or(Error::CreateError)?
            .text;

        let mut track_match_in_top_result: bool = false;

        if top_result_type == "Song" {
            if track.compare_similarity(
                &top_result.title.runs.get(0).ok_or(Error::CreateError)?.text,
                &top_result
                    .subtitle
                    .runs
                    .get(2)
                    .ok_or(Error::CreateError)?
                    .text,
                &top_result
                    .subtitle
                    .runs
                    .get(4)
                    .ok_or(Error::CreateError)?
                    .text,
                raw_duration_to_miliseconds(
                    &top_result
                        .subtitle
                        .runs
                        .get(6)
                        .ok_or(Error::CreateError)?
                        .text,
                )?,
            ) >= 3
            {
                track_match_in_top_result = true;
            }
        }

        let id: &str;
        let mut thumbnails: Vec<String> = Vec::new();

        if track_match_in_top_result == true {
            id = &top_result
                .title
                .runs
                .get(0)
                .ok_or(Error::CreateError)?
                .navigation_endpoint
                .as_ref()
                .ok_or(Error::CreateError)?
                .watch_endpoint
                .as_ref()
                .ok_or(Error::CreateError)?
                .video_id;

            for thumbnail in top_result
                .thumbnail
                .music_thumbnail_renderer
                .thumbnail
                .thumbnails
            {
                thumbnails.push(thumbnail.url.to_owned())
            }

            let artist_id: Option<String> = match &top_result
                .subtitle
                .runs
                .get(2)
                .ok_or(Error::CreateError)?
                .navigation_endpoint
            {
                Some(nav) => Some(
                    nav.browse_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .browse_id
                        .to_owned(),
                ),
                None => None,
            };

            let album_id: Option<String> = match &top_result
                .subtitle
                .runs
                .get(4)
                .ok_or(Error::CreateError)?
                .navigation_endpoint
            {
                Some(nav) => Some(
                    nav.browse_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .browse_id
                        .to_owned(),
                ),
                None => None,
            };

            let duration_raw: &str = &top_result
                .subtitle
                .runs
                .get(6)
                .ok_or(Error::CreateError)?
                .text;

            return Ok(Self {
                id: id.to_owned(),
                url: format!("https://www.youtube.com/watch?v={}", id),
                name: top_result
                    .title
                    .runs
                    .get(0)
                    .ok_or(Error::CreateError)?
                    .text
                    .to_owned(),
                artists: match artist_id {
                    Some(a_id) => vec![Artist {
                        id: a_id.to_owned(),
                        name: top_result
                            .subtitle
                            .runs
                            .get(2)
                            .ok_or(Error::CreateError)?
                            .text
                            .to_owned(),

                        url: format!("https://music.youtube.com/browse/{}", a_id),
                    }],
                    None => vec![],
                },
                album: match album_id {
                    Some(a_id) => Some(Album {
                        id: a_id.to_owned(),
                        name: top_result
                            .subtitle
                            .runs
                            .get(4)
                            .ok_or(Error::CreateError)?
                            .text
                            .to_owned(),
                        url: format!("https://music.youtube.com/browse/{}", a_id),
                        total_tracks: None,
                        ean: None,
                        upc: None,
                    }),
                    None => None,
                },
                duration_raw: duration_raw.to_owned(),
                duration_ms: raw_duration_to_miliseconds(duration_raw)?,
                music_video: None,
                thumbnails,
            });
        } else {
            let songs: RawMusicShelfRenderer = songs.ok_or(Error::CreateError)?;
            for song in songs.contents {
                let first_flex_run: &Vec<RawRun> = &song
                    .music_responsive_list_item_renderer
                    .flex_columns
                    .get(0)
                    .ok_or(Error::CreateError)?
                    .music_responsive_list_item_flex_column_renderer
                    .text
                    .runs;

                let second_flex_run: &Vec<RawRun> = &song
                    .music_responsive_list_item_renderer
                    .flex_columns
                    .get(1)
                    .ok_or(Error::CreateError)?
                    .music_responsive_list_item_flex_column_renderer
                    .text
                    .runs;

                if track.compare_similarity(
                    &first_flex_run.get(0).ok_or(Error::CreateError)?.text,
                    &second_flex_run.get(0).ok_or(Error::CreateError)?.text,
                    &second_flex_run.get(2).ok_or(Error::CreateError)?.text,
                    raw_duration_to_miliseconds(
                        &second_flex_run.get(4).ok_or(Error::CreateError)?.text,
                    )?,
                ) >= 3
                {
                    id = &first_flex_run
                        .get(0)
                        .ok_or(Error::CreateError)?
                        .navigation_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .watch_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .video_id;

                    for thumbnail in top_result
                        .thumbnail
                        .music_thumbnail_renderer
                        .thumbnail
                        .thumbnails
                    {
                        thumbnails.push(thumbnail.url.to_owned())
                    }

                    let artist_id: Option<String> = match &second_flex_run
                        .get(0)
                        .ok_or(Error::CreateError)?
                        .navigation_endpoint
                    {
                        Some(nav) => Some(
                            nav.browse_endpoint
                                .as_ref()
                                .ok_or(Error::CreateError)?
                                .browse_id
                                .to_owned(),
                        ),
                        None => None,
                    };

                    let album_id: Option<String> = match &second_flex_run
                        .get(2)
                        .ok_or(Error::CreateError)?
                        .navigation_endpoint
                    {
                        Some(nav) => Some(
                            nav.browse_endpoint
                                .as_ref()
                                .ok_or(Error::CreateError)?
                                .browse_id
                                .to_owned(),
                        ),
                        None => None,
                    };

                    let duration_raw: &str =
                        &second_flex_run.get(4).ok_or(Error::CreateError)?.text;

                    return Ok(Self {
                        id: id.to_owned(),
                        name: first_flex_run
                            .get(0)
                            .ok_or(Error::CreateError)?
                            .text
                            .to_owned(),
                        url: format!("https://www.youtube.com/watch?v={}", id),
                        artists: match artist_id {
                            Some(a_id) => vec![Artist {
                                id: a_id.to_owned(),
                                url: format!("https://music.youtube.com/browse/{}", a_id),
                                name: second_flex_run
                                    .get(0)
                                    .ok_or(Error::CreateError)?
                                    .text
                                    .to_owned(),
                            }],
                            None => vec![],
                        },
                        album: match album_id {
                            Some(a_id) => Some(Album {
                                id: a_id.to_owned(),
                                name: second_flex_run
                                    .get(2)
                                    .ok_or(Error::CreateError)?
                                    .text
                                    .to_owned(),
                                url: format!("https://music.youtube.com/browse/{}", a_id),
                                total_tracks: None,
                                ean: None,
                                upc: None,
                            }),
                            None => None,
                        },
                        duration_raw: duration_raw.to_owned(),
                        duration_ms: raw_duration_to_miliseconds(duration_raw)?,
                        music_video: None,
                        thumbnails,
                    });
                }
            }
        }

        // need to add music video support
        // let videos: RawMusicShelfRenderer = videos.ok_or(Error::CreateError)?;

        Err(Error::CreateError)
    }

    pub async fn download(&self, client: &Client, path: &str, filename: &str) -> Result<()> {
        match Command::new("yt-dlp")
            .arg(&self.url)
            .arg("-o")
            .arg(format!("{}{}.mp3", path, filename))
            .arg("-f")
            .arg("m4a")
            .stdout(Stdio::null())
            .status()
        {
            Ok(status) => {
                if status.success() == false {
                    return Err(Error::DownloadError);
                }
                // if self.thumbnails.len() > 0 {
                //     let image = match client.get(self.thumbnails[0]).send().await {
                //         Ok(response) => match response().await {
                //             Ok(bytes) => ,
                //             Err(..) =>
                //         },
                //         Err(..) => "".as_bytes()
                //     }
                // }
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(Error::DownloadError)
            }
        }
    }
}

fn raw_duration_to_miliseconds(raw_duration: &str) -> Result<usize> {
    let raw_parts = raw_duration.split(':');
    let mut base: usize = 1;
    let mut seconds: usize = 0;
    for part in raw_parts.rev() {
        let num: usize = match part.parse::<usize>() {
            Ok(n) => n,
            Err(..) => return Err(Error::CreateError),
        };

        seconds += num * base;
        base *= 60;
    }

    Ok(seconds * 1000)
}

#[cfg(test)]
mod tests {

    use crate::{service::Services, track::Track};

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
        };

        let client: reqwest::Client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                    .build()
                    .unwrap();

        example_track.add_youtube(&client).await.unwrap();
    }
}
