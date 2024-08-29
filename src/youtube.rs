use crate::error::{Error, Result};
use crate::service::{Album, Artist, Services};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct YouTube {
    pub id: String,
    pub url: String,
    pub music_video: Option<String>,
    pub image: Option<String>,
}

impl YouTube {
    const API_BASE_URL: &'static str = "https://www.youtube.com/youtubei/v1";
    const SITE_BASE_URL: &'static str = "https://www.youtube.com";

    pub async fn post(client: &Client, path: &str, body: &str) -> Result<serde_json::Value> {
        let request: RequestBuilder = client
            .post(format!("{}/{}", Self::API_BASE_URL, path))
            .body(body.to_owned()); // idk how to fix this with lifetimes....

        let response: Response = request.send().await?;
        if response.status() != 200 {
            eprintln!("{}", response.text().await?);
            return Err(Error::FindError);
        }

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    pub async fn get_raw_track_match_from_track(
        client: &Client,
        track: &Track,
    ) -> Result<serde_json::Value> {
        todo!()
    }

    pub async fn create_service_for_track(client: &Client, track: &mut Track) -> Result<()> {
        todo!()
    }

    pub async fn create_track_from_id(client: &Client, track_id: &str) -> Result<Track> {
        todo!()
    }

    pub async fn create_playlist_from_id(
        client: &Client,
        playlist_id: &str,
    ) -> Result<crate::track::Playlist> {
        todo!()
    }

    pub async fn download(&self, audio_only: bool) -> Result<()> {
        Ok(())
    }

    pub async fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    pub async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track> {
        todo!()
    }

    pub async fn create_playlist_from_raw(
        data: &Vec<serde_json::Value>,
    ) -> Result<crate::track::Playlist> {
        todo!()
    }
}
