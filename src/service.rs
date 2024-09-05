use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::error::Result;
use crate::spotify::Spotify;
use crate::track::{Playlist, Track};
use crate::youtube::YouTube;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Services {
    pub spotify: Option<Spotify>,
    pub apple_music: Option<AppleMusic>,
    pub youtube: Option<YouTube>,
    pub bandcamp: Option<Bandcamp>,
}

// pub trait Service<'a> {
//     const API_BASE_URL: &'static str;
//     const SITE_BASE_URL: &'static str;
//     // async fn get_raw_track_match_from_track(
//     //     client: &Client,
//     //     track: &Track,
//     // ) -> Result<serde_json::Value>;
// async fn create_service_for_track(client: &Client, track: &mut Track) -> Result<()>;
// async fn create_service_from_raw<T>(raw_track: &'a T) -> Result<Self>
// where
//     Self: Sized;
//     // async fn create_track_from_id(client: &Client, track_id: &str) -> Result<Track>;
// async fn create_track_from_raw(raw_track: &'a impl serde::Deserialize<'a>) -> Result<Track>;
//     // async fn create_playlist_from_id(client: &Client, playlist_id: &str) -> Result<Playlist>;
// async fn create_playlist_from_raw(
//     raw_playlist: &'a Vec<impl serde::Deserialize>,
// ) -> Result<Playlist>;
// }

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub total_tracks: Option<usize>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}
