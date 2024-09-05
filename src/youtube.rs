use crate::error::{Error, Result};
use crate::service::{Album, Artist};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct YouTube {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_raw: String,
    pub music_video: Option<String>,
    pub thumbnails: Vec<String>,
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
    const API_BASE_URL: &'static str = "https://music.youtube.com/youtubei/v1";
    const SITE_BASE_URL: &'static str = "https://music.youtube.com";
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

    pub async fn get_raw_track_match_from_track(client: &Client, track: &Track) -> Result<Value> {
        Ok(Self::get_raw_results_from_search(
            client,
            &format!(
                "{}, {}, {}, {}",
                track.name,
                track.artists.get(0).ok_or(Error::MatchError)?,
                track.release_year,
                track.album,
            ),
        )
        .await?["contents"]["tabbedSearchResultsRenderer"]["tabs"]
            .get_mut(0)
            .ok_or(Error::MatchError)?["tabRenderer"]["content"]["sectionListRenderer"]["contents"]
            .take())
    }

    pub async fn create_service_for_track(client: &Client, track: &mut Track) -> Result<()> {
        let data: Value = Self::get_raw_track_match_from_track(client, track).await?;
        let service: Self = Self::create_service_from_raw(&data, track).await?;
        track.services.youtube = Some(service);
        Ok(())
    }

    pub async fn create_service_from_raw(data: &Value, track: &Track) -> Result<Self> {
        let mut file = std::fs::File::create("./test.json")?;
        file.write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;

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
                    Err(e) => (),
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
            if top_result.title.runs.get(0).ok_or(Error::CreateError)?.text == track.name
                && top_result
                    .subtitle
                    .runs
                    .get(2)
                    .ok_or(Error::CreateError)?
                    .text
                    == *track.artists.get(0).ok_or(Error::CreateError)?
            {
                track_match_in_top_result = true;
            }
        }

        let id: &str;
        let music_video: &str;
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

            let artist_id: &str = &top_result
                .subtitle
                .runs
                .get(2)
                .ok_or(Error::CreateError)?
                .navigation_endpoint
                .as_ref()
                .ok_or(Error::CreateError)?
                .browse_endpoint
                .as_ref()
                .ok_or(Error::CreateError)?
                .browse_id;

            let album_id: &str = &top_result
                .subtitle
                .runs
                .get(4)
                .ok_or(Error::CreateError)?
                .navigation_endpoint
                .as_ref()
                .ok_or(Error::CreateError)?
                .browse_endpoint
                .as_ref()
                .ok_or(Error::CreateError)?
                .browse_id;

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
                artists: vec![Artist {
                    id: artist_id.to_owned(),
                    name: top_result
                        .subtitle
                        .runs
                        .get(2)
                        .ok_or(Error::CreateError)?
                        .text
                        .to_owned(),

                    url: format!("https://music.youtube.com/browse/{}", artist_id),
                }],
                album: Album {
                    id: album_id.to_owned(),
                    name: top_result
                        .subtitle
                        .runs
                        .get(4)
                        .ok_or(Error::CreateError)?
                        .text
                        .to_owned(),
                    url: format!("https://music.youtube.com/browse/{}", album_id),
                    total_tracks: None,
                    ean: None,
                    upc: None,
                },
                duration_raw: top_result
                    .subtitle
                    .runs
                    .get(6)
                    .ok_or(Error::CreateError)?
                    .text
                    .to_owned(),
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

                if first_flex_run.get(0).ok_or(Error::CreateError)?.text == track.name
                    && second_flex_run.get(0).ok_or(Error::CreateError)?.text
                        == *track.artists.get(0).ok_or(Error::CreateError)?
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

                    let artist_id: &str = &second_flex_run
                        .get(0)
                        .ok_or(Error::CreateError)?
                        .navigation_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .browse_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .browse_id;

                    let album_id: &str = &second_flex_run
                        .get(2)
                        .ok_or(Error::CreateError)?
                        .navigation_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .browse_endpoint
                        .as_ref()
                        .ok_or(Error::CreateError)?
                        .browse_id;

                    return Ok(Self {
                        id: id.to_owned(),
                        name: first_flex_run
                            .get(0)
                            .ok_or(Error::CreateError)?
                            .text
                            .to_owned(),
                        url: format!("https://www.youtube.com/watch?v={}", id),
                        artists: vec![Artist {
                            id: artist_id.to_owned(),
                            url: format!("https://music.youtube.com/browse/{}", artist_id),
                            name: second_flex_run
                                .get(0)
                                .ok_or(Error::CreateError)?
                                .text
                                .to_owned(),
                        }],
                        album: Album {
                            id: album_id.to_owned(),
                            name: second_flex_run
                                .get(2)
                                .ok_or(Error::CreateError)?
                                .text
                                .to_owned(),
                            url: format!("https://music.youtube.com/browse/{}", album_id),
                            total_tracks: None,
                            ean: None,
                            upc: None,
                        },
                        duration_raw: second_flex_run
                            .get(4)
                            .ok_or(Error::CreateError)?
                            .text
                            .to_owned(),
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
}
