use crate::error::{Error, Result};
use crate::service::{Album, Artist, Service, Services};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AppleMusic {
    pub id: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub url: String,
    pub image: Option<String>,
    pub genres: Vec<String>,
    pub audio_preivew: Option<String>,
}

impl AppleMusic {
    pub const PUBLIC_BEARER_TOKEN: &str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldlYlBsYXlLaWQifQ.eyJpc3MiOiJBTVBXZWJQbGF5IiwiaWF0IjoxNzIxNzczNjI0LCJleHAiOjE3MjkwMzEyMjQsInJvb3RfaHR0cHNfb3JpZ2luIjpbImFwcGxlLmNvbSJdfQ.cMMhHLLazlxgiIbwBSSP1YuHCgqAVxiF7UQrwBc5xZepWt-vjqth_o4BidXFrmsEJvwzZKJ-GAMbqJpIeGcl7w";
}

impl Service for AppleMusic {
    const API_BASE_URL: &'static str = "https://api.music.apple.com/v1";
    // const API_BASE_URL: &'static str = "https://amp-api-edge.music.apple.com/v1";
    const SITE_BASE_URL: &'static str = "https://music.apple.com";

    async fn get_raw_track_match_from_search(
        client: &Client,
        auth_token: &str,
        query: &str,
    ) -> Result<serde_json::Value> {
        todo!()
    }

    async fn get_raw_track_match_from_track(
        client: &Client,
        auth_token: &str,
        track: &Track,
    ) -> Result<serde_json::Value> {
        todo!()
    }

    async fn create_service_for_track(
        client: &Client,
        auth_token: &str,
        track: &mut Track,
    ) -> Result<()> {
        todo!()
    }

    async fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    async fn create_track_from_id(
        client: &Client,
        auth_token: &str,
        track_id: &str,
    ) -> Result<Track> {
        todo!()
    }

    async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track> {
        todo!()
    }

    async fn create_playlist_from_id(
        client: &Client,
        auth_token: &str,
        playlist_id: &str,
    ) -> Result<crate::track::Playlist> {
        todo!()
    }

    async fn create_playlist_from_raw(data: &serde_json::Value) -> Result<crate::track::Playlist> {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     // use crate::apple_music::AppleMusic;

//     #[test]
//     fn test_isrc_three_minute_hero_by_the_selector() {
//         dotenv::dotenv().ok();
//         let results = search_by_isrc("GBAYK8000001");
//         // assert!(t.n > 0)
//     }
// }
