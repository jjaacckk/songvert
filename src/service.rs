use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::error::Result;
use crate::spotify::Spotify;
use crate::track::{Playlist, Track};
use crate::youtube::YouTube;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Services {
    pub spotify: Option<Spotify>,
    pub apple_music: Option<AppleMusic>,
    pub youtube: Option<YouTube>,
    pub bandcamp: Option<Bandcamp>,
}

pub trait Service {
    const API_BASE_URL: &'static str;
    const SITE_BASE_URL: &'static str;
    async fn get_raw_track_match_from_track(
        client: &Client,
        auth: &Option<&str>,
        track: &Track,
    ) -> Result<serde_json::Value>;
    async fn create_service_for_track(
        client: &Client,
        auth: &Option<&str>,
        track: &mut Track,
    ) -> Result<()>;
    async fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
    where
        Self: Sized;
    async fn create_track_from_id(
        client: &Client,
        auth: &Option<&str>,
        track_id: &str,
    ) -> Result<Track>;
    async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track>;
    async fn create_playlist_from_id(
        client: &Client,
        auth: &Option<&str>,
        playlist_id: &str,
    ) -> Result<Playlist>;
    async fn create_playlist_from_raw(data: &serde_json::Value) -> Result<Playlist>;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub total_tracks: u8,
    pub ean: Option<String>,
    pub upc: Option<String>,
}
