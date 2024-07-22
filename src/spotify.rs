use crate::service::{AlbumOnService, ArtistOnService, Service};
use crate::track::Track;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
struct SessionInfo {
    access_token: String,
    access_token_expiration_timestamp_ms: String,
    is_anonymous: bool,
    client_id: String,
}

impl Spotify {
    fn grab_public_api_session_info(client: &Client) -> SessionInfo {
        client.get()
    }
}

impl Service for Spotify {
    fn search_by_isrc(isrc: &str) -> serde_json::Value {
        let search_data: serde_json::Value;
        search_data
    }

    fn search_by_name(name: &str) -> serde_json::Value {}

    fn get_raw_match(track: &Track) -> serde_json::Value {
        match track.isrc {
            Some(isrc) => data = Self::search_by_isrc(&isrc),
            None => {}
        }
    }

    fn add_service_to_track(track: &mut Track) -> bool {
        let data: serde_json::Value = Self::get_raw_match(track);
        let service: Spotify = Self::create_service_from_raw(&data);
        track.services.spotify = Some(service);
        true
    }

    fn create_service_from_raw(data: &serde_json::Value) -> Spotify {
        Spotify {
            id: (),
            artists: (),
            album: (),
            url: (),
            image: (),
            audio_preview: (),
        }
    }

    fn create_track_from_raw(data: &serde_json::Value) -> Track {
        Track {
            title: (),
            album: (),
            disk_number: (),
            track_number: (),
            artists: (),
            release_year: (),
            release_month: (),
            release_day: (),
            is_explicit: (),
            duration_ms: (),
            services: (), // only adds own service (spotify)
            isrc: (),
            ean: (),
            upc: (),
        }
    }

    fn create_track_by_id(id: &str) -> Track {
        let track_data: serde_json::Value;
        Self::create_track_from_raw(track_data)
    }
    fn create_tracks_from_playlist_raw(data: &serde_json::Value) -> Vec<Track> {
        let new_tracks: Vec<Track> = Vec::new();
        new_tracks
    }

    fn create_tracks_from_playlist_id(id: &str) -> Vec<Track> {
        let playlist_data: serde_json::Value;
        Self::create_tracks_from_playlist_raw(playlist_data)
    }
}
