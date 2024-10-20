use crate::apple_music::AppleMusic;
use crate::error::Result;
use crate::service::Source;
use crate::spotify::Spotify;
use crate::track::Track;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
    pub id: String,
    pub description: Option<String>,
    pub source_service: Source,
}

impl Playlist {
    pub async fn download_tracks(
        &self,
        client: &Client,
        download_path: &Path,
        add_metadata: bool,
    ) -> Result<()> {
        println!(
            "Attempting to download {} tracks for playlist {} to {}",
            self.tracks.len(),
            self.name,
            download_path.to_string_lossy()
        );

        std::fs::create_dir_all(&download_path)?;

        let mut count = 1;
        let mut track_names: Vec<String> = Vec::with_capacity(self.tracks.len());
        let mut download_futures = Vec::with_capacity(self.tracks.len());

        for track in &self.tracks {
            track_names.push(format!(
                "({}) {} - {}",
                count,
                track.name,
                track.artists.join(", ")
            ));
            count += 1;
        }

        count = 0;
        for track in &self.tracks {
            download_futures.push(track.download(
                client,
                download_path,
                &track_names[count],
                add_metadata,
            ));
            count += 1;
        }

        futures::future::join_all(download_futures).await;
        Ok(())
    }

    pub fn save_to_file(&self, playlist_file_path: &Path, playlist_filename: &str) -> Result<()> {
        let mut full_path: PathBuf = playlist_file_path.to_owned();
        full_path.push(playlist_filename);
        full_path.set_extension("json");

        println!(
            "Attempting to save playlist data to {}",
            full_path.to_string_lossy()
        );
        std::fs::create_dir_all(playlist_file_path)?;
        let mut playlist_file = std::fs::File::create(full_path)?;
        playlist_file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;

        Ok(())
    }
    pub async fn add_spotify(&mut self, client: &Client, auth: &str) -> Result<()> {
        let mut spotify_service_futures = Vec::with_capacity(self.tracks.len());
        for track in &mut self.tracks {
            spotify_service_futures.push(track.add_spotify(client, auth));
        }
        let results = futures::future::join_all(spotify_service_futures).await;
        let mut count = 1;
        for result in results {
            match result {
                Ok(..) => (),
                Err(e) => {
                    println!("\tSkipping adding Spotify to track {}: {}", count + 1, e)
                }
            };
            count += 1;
        }

        Ok(())
    }
    pub async fn add_apple_music(&mut self, client: &Client, auth: &str) -> Result<()> {
        let mut apple_music_service_futures = Vec::with_capacity(self.tracks.len());
        for track in &mut self.tracks {
            apple_music_service_futures.push(track.add_apple_music(client, auth));
        }
        let results = futures::future::join_all(apple_music_service_futures).await;
        let mut count = 1;
        for result in results {
            match result {
                Ok(..) => (),
                Err(e) => {
                    println!(
                        "\tSkipping adding Apple Music to track {}: {}",
                        count + 1,
                        e
                    )
                }
            };
            count += 1;
        }

        Ok(())
    }

    pub async fn add_youtube(&mut self, client: &Client) -> Result<()> {
        let mut youtube_service_futures = Vec::with_capacity(self.tracks.len());
        for track in &mut self.tracks {
            youtube_service_futures.push(track.add_youtube(client));
        }
        let results = futures::future::join_all(youtube_service_futures).await;
        let mut count = 1;
        for result in results {
            match result {
                Ok(..) => (),
                Err(e) => {
                    println!("\tSkipping adding YouTube to track {}: {}", count, e)
                }
            };
            count += 1;
        }

        Ok(())
    }

    pub async fn add_bandcamp(&mut self, client: &Client) -> Result<()> {
        let mut bandcamp_service_futures = Vec::with_capacity(self.tracks.len());
        for track in &mut self.tracks {
            bandcamp_service_futures.push(track.add_bandcamp(client));
        }
        let results = futures::future::join_all(bandcamp_service_futures).await;
        let mut count = 1;
        for result in results {
            match result {
                Ok(..) => (),
                Err(e) => {
                    println!("\tSkipping adding Bandcamp to track {}: {}", count, e)
                }
            };
            count += 1;
        }
        Ok(())
    }

    pub fn from_file(file_path: &Path) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(file_path)?)?)
    }

    pub async fn from_spotify_id(
        client: &Client,
        spotify_auth: &str,
        spotify_playlist_id: &str,
    ) -> Result<Self> {
        let playlist: Self =
            Spotify::create_playlist_from_id(client, spotify_auth, spotify_playlist_id).await?;

        Ok(playlist)
    }

    pub async fn from_apple_music_id(
        client: &Client,
        apple_music_auth: &str,
        apple_music_playlist_id: &str,
    ) -> Result<Self> {
        let playlist: Playlist =
            AppleMusic::create_playlist_from_id(client, apple_music_auth, apple_music_playlist_id)
                .await?;

        Ok(playlist)
    }
}
