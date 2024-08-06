// use crate::service::{AlbumOnService, ArtistOnService, Services};
// use crate::track::Track;
// use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct YouTube {
    pub id: String,
    pub url: String,
    pub music_video: Option<String>,
    pub image: Option<String>,
}
