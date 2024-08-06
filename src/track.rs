use crate::service::Services;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Track {
    pub name: String,
    pub album: String,
    pub disk_number: u8,
    pub track_number: u8,
    pub artists: Vec<String>,
    pub release_year: u16,
    pub release_month: Option<u8>,
    pub release_day: Option<u8>,
    pub is_explicit: bool,
    pub duration_ms: u64,
    pub services: Services,
    pub isrc: Option<String>,
}

pub type Playlist = Vec<Track>;

#[cfg(test)]
mod tests {

    // use crate::apple_music::AppleMusic;
    // use crate::bandcamp::Bandcamp;
    use crate::service::{Album, Artist, Service, Services};
    use crate::spotify::{SessionInfo, Spotify};
    use crate::track::Track;
    // use crate::youtube::YouTube;

    #[tokio::test]
    async fn example_track_data_insertion() {
        let example_spotify_service: Spotify = Spotify {
            id: String::from("6K225HZ3V7F4ec7yi1o88C"),
            artists: vec![Artist {id: String::from("0xiwsYZwhrizQGNaQtW942"), name: String::from("Tunabunny")}],
            album: Album { id: String::from("6WSL47W7Z5WwCCKzaFyLGd"), name: String::from("Genius Fatigue"), total_tracks: 10, ean: None, upc: None},
            url: String::from("https://open.spotify.com/track/6K225HZ3V7F4ec7yi1o88C"),
            image: Some(String::from("https://i.scdn.co/image/ab67616d0000b27336a71c545ed453f80433f6c8")),
            audio_preview: Some(String::from("https://p.scdn.co/mp3-preview/13a7bfeabbe56d852fb9f7b6291c7dc49bcde515?cid=d8a5ed958d274c2e8ee717e6a4b0971d")),
        };

        let example_services: Services = Services {
            spotify: Some(example_spotify_service),
            apple_music: None,
            youtube: None,
            bandcamp: None,
        };

        let example_track: Track = Track {
            name: String::from("Duchess for Nothing"),
            album: String::from("Genius Fatigue"),
            disk_number: 1,
            track_number: 1,
            artists: vec![String::from("Tunabunny")],
            release_year: 2013,
            release_month: None,
            release_day: None,
            is_explicit: false,
            duration_ms: 138026,
            services: example_services,
            isrc: Some(String::from("USZUD1215001")),
        };
        // println!(
        //     "{}",
        //     serde_json::to_string_pretty(&example_track).unwrap_or("".to_string())
        // );
        //
        let client: reqwest::Client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                .build()
                .unwrap();
        let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();
        assert_eq!(
            example_track,
            Spotify::create_track_from_id(
                &client,
                &session_info.access_token,
                "6K225HZ3V7F4ec7yi1o88C"
            )
            .await
            .unwrap()
        );
    }
}
