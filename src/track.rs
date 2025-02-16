use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::error::{Error, Result};
use crate::service::{Services, Source};
use crate::spotify::Spotify;
use crate::utils::{add_metadata_to_m4a, add_metadata_to_mp3};
use crate::youtube::YouTube;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Track {
    pub name: String,
    pub album: String,
    pub disk_number: usize,
    pub track_number: usize,
    pub artists: Vec<String>,
    pub release_year: usize,
    pub release_month: Option<usize>,
    pub release_day: Option<usize>,
    pub is_explicit: bool,
    pub duration_ms: usize,
    pub isrc: Option<String>,
    pub services: Services,
    pub source_service: Source,
}

impl Track {
    pub async fn add_spotify(&mut self, client: &Client, auth: &str) -> Result<()> {
        Spotify::create_service_for_track(client, auth, self).await?;

        Ok(())
    }

    pub async fn add_apple_music(&mut self, client: &Client, auth: &str) -> Result<()> {
        AppleMusic::create_service_for_track(client, auth, self).await?;

        Ok(())
    }

    pub async fn add_youtube(&mut self, client: &Client) -> Result<()> {
        YouTube::create_service_for_track(client, self).await?;

        Ok(())
    }

    pub async fn add_bandcamp(&mut self, client: &Client) -> Result<()> {
        Bandcamp::create_service_for_track(client, self).await?;

        Ok(())
    }

    pub fn compare_similarity_fuzzy(
        &self,
        compare_name: &str,
        compare_artist: &str,
        compare_album: &str,
        compare_duration_ms: usize,
    ) -> f64 {
        //println!(
        //    "{} <=> {}\n{} <=> {}\n{} <=> {}\n{} <=> {}",
        //    self.name,
        //    compare_name,
        //    self.artists[0],
        //    compare_artist,
        //    self.album,
        //    compare_album,
        //    self.duration_ms,
        //    compare_duration_ms
        //);

        let mut count: f64 = 0.0;

        count += strsim::jaro_winkler(&self.name.to_lowercase(), &compare_name.to_lowercase());
        count += strsim::jaro_winkler(&self.album.to_lowercase(), &compare_album.to_lowercase());

        if self.artists.len() > 0 {
            count += strsim::jaro_winkler(
                &self.artists[0].to_lowercase(),
                &compare_artist.to_lowercase(),
            );
        }

        if compare_duration_ms.abs_diff(self.duration_ms) <= 3000 {
            // no more than 3 second difference
            count += 1.0;
        }
        //println!("score: {}", count);
        count
    }

    pub fn compare_similarity(
        &self,
        compare_name: &str,
        compare_artist: &str,
        compare_album: &str,
        compare_duration_ms: usize,
    ) -> u8 {
        let mut count = 0;
        if compare_name.to_lowercase() == self.name.to_lowercase() {
            count += 1;
        }

        if compare_album.to_lowercase() == self.album.to_lowercase() {
            count += 1;
        }

        if self.artists.len() > 0 {
            if compare_artist.to_lowercase() == self.artists[0].to_lowercase() {
                count += 1;
            }
        }

        if compare_duration_ms.abs_diff(self.duration_ms) <= 3000 {
            // no more than 3 second difference
            count += 1;
        }

        count
    }

    pub async fn download(
        &self,
        client: &Client,
        path: &Path,
        filename: &str,
        add_metadata: bool,
    ) -> Result<()> {
        if let Some(bandcamp) = &self.services.bandcamp {
            if let Ok(download_path) = bandcamp.download(client, path, filename).await {
                if add_metadata == true {
                    add_metadata_to_mp3(client, &download_path, &self, false).await?;
                }
                return Ok(());
            }
        }

        if let Some(youtube) = &self.services.youtube {
            if let Ok(download_path) = youtube.download(path, filename).await {
                if add_metadata == true {
                    add_metadata_to_m4a(client, &download_path, &self, false).await?;
                }
                return Ok(());
            }
        }

        eprintln!(
            "\tSkipping downloading track {} - {}",
            self.name,
            self.artists.join(", ")
        );
        Err(Error::DownloadError("download failed".to_string()))
    }

    pub async fn from_spotify_id(
        client: &Client,
        spotify_auth: &str,
        spotify_track_id: &str,
    ) -> Result<Self> {
        let track: Self =
            Spotify::create_track_from_id(client, spotify_auth, spotify_track_id).await?;

        Ok(track)
    }

    pub async fn from_apple_music_id(
        client: &Client,
        apple_music_auth: &str,
        apple_music_track_id: &str,
    ) -> Result<Self> {
        let track: Self =
            AppleMusic::create_track_from_id(client, apple_music_auth, apple_music_track_id)
                .await?;

        Ok(track)
    }

    pub fn from_file(file_path: &Path) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(file_path)?)?)
    }

    pub fn save_to_file(&self, track_file_path: &Path, track_filename: &str) -> Result<()> {
        let mut full_path: PathBuf = track_file_path.to_owned();
        full_path.push(track_filename);
        full_path.set_extension("json");

        println!(
            "Attempting to save track data to {}",
            full_path.to_string_lossy()
        );
        std::fs::create_dir_all(track_file_path)?;
        let mut track_file = std::fs::File::create(full_path)?;
        track_file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
