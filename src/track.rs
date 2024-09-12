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
    pub async fn add_spotify(&mut self, auth: &str, client: &Client) -> Result<()> {
        Spotify::create_service_for_track(client, auth, self).await?;
        // println!("sp done for {} by {}", self.name, self.artists[0]);
        Ok(())
    }

    pub async fn add_apple_music(&mut self, auth: &str, client: &Client) -> Result<()> {
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
}

// pub type Playlist = [Track];
pub type Playlist = Vec<Track>;

#[cfg(test)]
mod tests {

    // // use crate::apple_music::AppleMusic;
    // // use crate::bandcamp::Bandcamp;
    // use crate::service::{Album, Artist, Services};
    // use crate::spotify::{SessionInfo, Spotify};
    // use crate::track::Track;
    // // use crate::youtube::YouTube;

    // #[tokio::test]
    // async fn example_track_data_insertion() {
    //     let example_spotify_service: Spotify = Spotify {
    //         id: "6K225HZ3V7F4ec7yi1o88C".to_owned(),
    //         artists: vec![Artist {id: "0xiwsYZwhrizQGNaQtW942".to_owned(), name: "Tunabunny".to_owned()}],
    //         album: Album { id: "6WSL47W7Z5WwCCKzaFyLGd".to_owned(), name: "Genius Fatigue".to_owned(), total_tracks: 10, ean: None, upc: None},
    //         url: "https://open.spotify.com/track/6K225HZ3V7F4ec7yi1o88C".to_owned(),
    //         image: Some("https://i.scdn.co/image/ab67616d0000b27336a71c545ed453f80433f6c8".to_owned()),
    //         audio_preview: Some("https://p.scdn.co/mp3-preview/13a7bfeabbe56d852fb9f7b6291c7dc49bcde515?cid=d8a5ed958d274c2e8ee717e6a4b0971d".to_owned()),
    //     };

    //     let example_services: Services = Services {
    //         spotify: Some(example_spotify_service),
    //         apple_music: None,
    //         youtube: None,
    //         bandcamp: None,
    //     };

    //     let example_track: Track = Track {
    //         name: "Duchess for Nothing".to_owned(),
    //         album: "Genius Fatigue".to_owned(),
    //         disk_number: 1,
    //         track_number: 1,
    //         artists: Vec::from(["Tunabunny".to_owned()]),
    //         release_year: 2013,
    //         release_month: None,
    //         release_day: None,
    //         is_explicit: false,
    //         duration_ms: 138026,
    //         services: example_services,
    //         isrc: Some("USZUD1215001".to_owned()),
    //     };
    //     // println!(
    //     //     "{}",
    //     //     serde_json::to_string_pretty(&example_track).unwrap_or("".to_string())
    //     // );
    //     //
    //     let client: reqwest::Client = reqwest::Client::builder()
    //             .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
    //             .build()
    //             .unwrap();
    //     let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();
    //     assert_eq!(
    //         example_track,
    //         Spotify::create_track_from_id(
    //             &client,
    //             &session_info.access_token,
    //             "6K225HZ3V7F4ec7yi1o88C"
    //         )
    //         .await
    //         .unwrap()
    //     );
    // }
}
