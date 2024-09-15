use crate::apple_music::AppleMusic;
use crate::bandcamp::Bandcamp;
use crate::error::{Error, Result};
use crate::service::{Album, Artist, Services};
use crate::spotify::Spotify;
use crate::youtube::YouTube;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
}

impl Track {
    pub async fn add_spotify(&mut self, client: &Client, auth: &str) -> Result<()> {
        Spotify::create_service_for_track(client, auth, self).await?;
        // println!("sp done for {} by {}", self.name, self.artists[0]);
        Ok(())
    }

    pub async fn add_apple_music(&mut self, client: &Client, auth: &str) -> Result<()> {
        AppleMusic::create_service_for_track(client, auth, self).await?;
        // println!("am done for {} by {}", self.name, self.artists[0]);
        Ok(())
    }

    pub async fn add_youtube(&mut self, client: &Client) -> Result<()> {
        YouTube::create_service_for_track(client, self).await?;
        // println!("yt done for {} by {}", self.name, self.artists[0]);
        Ok(())
    }

    pub async fn add_bandcamp(&mut self, client: &Client) -> Result<()> {
        Bandcamp::create_service_for_track(client, self).await?;
        // println!("bc done for {} by {}", self.name, self.artists[0]);
        Ok(())
    }

    pub fn compare_similarity(
        &self,
        compare_name: &str,
        compare_artist: &str,
        compare_album: &str,
        compare_duration_ms: usize,
    ) -> u8 {
        let mut count = 0;

        // println!(
        //     "{} =?= {}",
        //     compare_name.to_lowercase(),
        //     self.name.to_lowercase()
        // );
        if compare_name.to_lowercase() == self.name.to_lowercase() {
            count += 1;
        }

        // println!(
        //     "{} =?= {}",
        //     compare_album.to_lowercase(),
        //     self.album.to_lowercase()
        // );
        if compare_album.to_lowercase() == self.album.to_lowercase() {
            count += 1;
        }

        // println!(
        //     "{} =?= {}",
        //     compare_artist.to_lowercase(),
        //     self.artists[0].to_lowercase()
        // );
        if self.artists.len() > 0 {
            if compare_artist.to_lowercase() == self.artists[0].to_lowercase() {
                count += 1;
            }
        }

        // println!("{}", compare_duration_ms.abs_diff(self.duration_ms));
        if compare_duration_ms.abs_diff(self.duration_ms) <= 3000 {
            // no more than 3 second difference
            count += 1;
        }

        count
    }

    pub async fn download(&self, client: &Client, filename: &str, path: &str) -> Result<()> {
        match &self.services.bandcamp {
            Some(bandcamp) => match bandcamp.download(client, filename, path).await {
                Ok(..) => return Ok(()),
                // Err(e) => eprintln!("\t*bandcamp download failed for {}: {}", self.name, e),
                Err(..) => ()
            },
            // None => eprintln!("\t*no bandcamp for {}", self.name),
            None => ()
        };

        match &self.services.youtube {
            Some(youtube) => match youtube.download(client, filename, path).await {
                Ok(..) => return Ok(()),
                // Err(e) => eprintln!("\t*youtube download failed for {}: {}", self.name, e),
                Err(..) => ()
            },
            // None => eprintln!("\t*no youtube for {}", self.name),
            None => ()
        };

        eprintln!(
            "\tSkipping downloading track {} - {}",
            self.name,
            self.artists.join(", ")
        );
        Err(Error::DownloadError)
    }
}

#[cfg(test)]
mod tests {}
