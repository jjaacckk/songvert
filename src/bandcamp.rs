use crate::error::{Error, Result};
use crate::service::{Album, Artist};
use crate::track::Track;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Bandcamp {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub image: Option<String>,
    pub audio_file: Option<String>,
}

impl Bandcamp {
    pub async fn download(&self) -> Result<()> {
        Ok(())
    }
}

fn search_by_name() {}

fn download_track() {}
