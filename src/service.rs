use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::error::{Error, Result};
use crate::spotify::Spotify;
use crate::track::Track;
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
    // fn search_by_isrc(isrc: &str) -> Result<serde_json::Value, serde_json::Error>;
    // fn search_by_name(name: &str) -> Result<serde_json::Value, serde_json::Error>;
    async fn get_raw_track_match(
        client: &Client,
        auth_token: &str,
        track: &Track,
    ) -> Result<serde_json::Value>;
    async fn add_service_to_track(
        client: &Client,
        auth_token: &str,
        track: &mut Track,
    ) -> Result<()>;
    async fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
    where
        Self: Sized;
    async fn create_track_from_id(
        client: &Client,
        auth_token: &str,
        track_id: &str,
    ) -> Result<Track>;
    async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track>;
    async fn create_tracks_from_playlist_id(
        client: &Client,
        auth_token: &str,
        playlist_id: &str,
    ) -> Result<Vec<Track>>;
    async fn create_tracks_from_playlist_raw(data: &serde_json::Value) -> Result<Vec<Track>>;
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
