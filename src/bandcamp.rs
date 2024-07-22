use crate::service::{AlbumOnService, ArtistOnService};
// use crate::track::Track;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Bandcamp {
    pub id: String,
    pub url: String,
    pub artists: Vec<ArtistOnService>,
    pub album: AlbumOnService,
    pub image: Option<String>,
    pub audio_file: Option<String>,
}

fn search_by_name() {}

fn download_track() {}
