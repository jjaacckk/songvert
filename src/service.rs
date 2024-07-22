use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
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
    fn search_by_isrc(isrc: &str) -> serde_json::Value;
    fn search_by_name(name: &str) -> serde_json::Value;
    fn get_raw_match(track: &Track) -> serde_json::Value;
    fn add_service_to_track(track: &mut Track) -> bool;
    fn create_service_from_raw(data: &serde_json::Value) -> Self;
    fn create_track_from_raw(data: &serde_json::Value) -> Track;
    fn create_track_by_id(id: &str) -> Track;
    fn create_tracks_from_playlist_id(id: &str) -> Vec<Track>;
    fn create_tracks_from_playlist_raw(data: &serde_json::Value) -> Vec<Track>;
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
