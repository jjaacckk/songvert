use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::error::{Error, Result};
use crate::spotify::Spotify;
use crate::track::Track;
use crate::youtube::YouTube;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Services {
    pub spotify: Option<Spotify>,
    pub apple_music: Option<AppleMusic>,
    pub youtube: Option<YouTube>,
    pub bandcamp: Option<Bandcamp>,
}
pub trait Service {
    // fn search_by_isrc(isrc: &str) -> Result<serde_json::Value, serde_json::Error>;
    // fn search_by_name(name: &str) -> Result<serde_json::Value, serde_json::Error>;
    fn get_raw_track_match(track: &Track) -> Result<serde_json::Value>;
    fn add_service_to_track(track: &mut Track) -> Result<()>;
    fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
    where
        Self: Sized;
    fn create_track_from_id(id: &str) -> Result<Track>;
    fn create_track_from_raw(data: &serde_json::Value) -> Result<Track>;
    fn create_tracks_from_playlist_id(id: &str) -> Result<Vec<Track>>;
    fn create_tracks_from_playlist_raw(data: &serde_json::Value) -> Result<Vec<Track>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtistOnService {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbumOnService {
    pub id: String,
    pub name: String,
}
