use crate::error::SpotifyError;
use crate::service::{AlbumOnService, ArtistOnService, Service};
use regex::Regex;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RawTrack {}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RawPlaylist {}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RawAlbum {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spotify {
    pub id: String,
    pub artists: Vec<ArtistOnService>,
    pub album: AlbumOnService,
    pub url: String,
    pub image: Option<String>,
    pub audio_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
// #[serde(deny_unknown_fields, rename_all(deserialize = "snake_case"))]
struct SessionInfo {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "accessTokenExpirationTimestampMs")]
    access_token_expiration_timestamp_ms: u64,
    #[serde(rename = "isAnonymous")]
    is_anonymous: bool,
    #[serde(rename = "clientId")]
    client_id: String,
}

impl Spotify {
    async fn get_public_api_session_info(client: &Client) -> Result<SessionInfo, SpotifyError> {
        let request: RequestBuilder = client.get("https://open.spotify.com");
        let response: Response = match request.send().await {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };

        if response.status() != 200 {
            panic!(
                "{} status error when fetching session info",
                response.status()
            )
        }

        let raw_html: String = response.text().await.unwrap();

        let re = Regex::new(r#"(\{"accessToken":.*"})"#).unwrap();
        let captures = re
            .captures(&raw_html)
            .expect("no access token found in HTML");

        let session_info: SessionInfo =
            serde_json::from_str(captures.get(0).unwrap().as_str()).unwrap();

        Ok(session_info)
    }

    // fn search_by_isrc(isrc: &str) -> serde_json::Value {
    //     let search_data: serde_json::Value;
    //     search_data
    // }

    // fn search_by_name(name: &str) -> serde_json::Value {}
}

// impl Service for Spotify {
//     fn get_raw_match(track: &Track) -> serde_json::Value {
//         match track.isrc {
//             Some(isrc) => data = Self::search_by_isrc(&isrc),
//             None => {}
//         }
//     }

//     fn add_service_to_track(track: &mut Track) -> bool {
//         let data: serde_json::Value = Self::get_raw_match(track);
//         let service: Spotify = Self::create_service_from_raw(&data);
//         track.services.spotify = Some(service);
//         true
//     }

//     fn create_service_from_raw(data: &serde_json::Value) -> Spotify {
//         Spotify {
//             id: (),
//             artists: (),
//             album: (),
//             url: (),
//             image: (),
//             audio_preview: (),
//         }
//     }

//     fn create_track_from_raw(data: &serde_json::Value) -> Track {
//         Track {
//             title: (),
//             album: (),
//             disk_number: (),
//             track_number: (),
//             artists: (),
//             release_year: (),
//             release_month: (),
//             release_day: (),
//             is_explicit: (),
//             duration_ms: (),
//             services: (), // only adds own service (spotify)
//             isrc: (),
//             ean: (),
//             upc: (),
//         }
//     }

//     fn create_track_from_id(id: &str) -> Track {
//         let track_data: serde_json::Value;
//         Self::create_track_from_raw(track_data)
//     }
//     fn create_tracks_from_playlist_raw(data: &serde_json::Value) -> Vec<Track> {
//         let new_tracks: Vec<Track> = Vec::new();
//         new_tracks
//     }

//     fn create_tracks_from_playlist_id(id: &str) -> Vec<Track> {
//         let playlist_data: serde_json::Value;
//         Self::create_tracks_from_playlist_raw(playlist_data)
//     }
// }

#[cfg(test)]
mod tests {

    use crate::spotify::{SessionInfo, Spotify};

    #[tokio::test]
    async fn get_session_info() {
        let client: reqwest::Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()
            .unwrap();
        let session_info: SessionInfo =
            Spotify::get_public_api_session_info(&client).await.unwrap();
        println!("{:?}", session_info);
    }
}
