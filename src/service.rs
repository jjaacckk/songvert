use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::spotify::Spotify;
use crate::youtube::YouTube;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Services {
    pub spotify: Option<Spotify>,
    pub apple_music: Option<AppleMusic>,
    pub youtube: Option<YouTube>,
    pub bandcamp: Option<Bandcamp>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum Source {
    Spotify,
    AppleMusic,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub url: String,
    pub total_tracks: Option<usize>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}
