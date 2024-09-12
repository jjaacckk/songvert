use crate::error::{Error, Result};
use crate::service::{Album, Artist, Services};
use crate::track::{Playlist, Track};
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Spotify {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_ms: usize,
    pub image: Option<String>,
    pub audio_preview: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawPlaylist {
    collaborative: bool,
    description: Option<String>,
    external_urls: RawExternalUrls,
    followers: Value,
    href: String,
    id: String,
    images: Vec<RawImage>,
    name: String,
    owner: Value,
    primary_color: Option<String>,
    public: Option<bool>,
    snapshot_id: String,
    tracks: RawPlaylistTracks,
    r#type: String,
    uri: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawPlaylistTracks {
    href: String,
    items: Vec<RawPlaylistTrackItem>,
    limit: usize,
    next: Option<String>,
    offset: usize,
    previous: Option<String>,
    total: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawPlaylistTrackItem {
    added_at: String,
    added_by: Value,
    is_local: bool,
    primary_color: Option<String>,
    track: RawTrack,
    // video_thumbnail: Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawTrack {
    preview_url: Option<String>,
    available_markets: Value,
    explicit: bool,
    r#type: String,
    album: RawTrackAlbum,
    artists: Vec<RawTrackArtist>,
    disc_number: usize,
    track_number: usize,
    duration_ms: usize,
    external_ids: RawTrackExternalIds,
    external_urls: RawExternalUrls,
    href: String,
    id: String,
    name: String,
    popularity: usize,
    uri: String,
    is_local: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawTrackAlbum {
    available_markets: Value,
    r#type: String,
    album_type: String,
    href: String,
    id: String,
    images: Vec<RawImage>,
    name: String,
    release_date: String,
    release_date_precision: String,
    uri: String,
    artists: Vec<RawTrackArtist>,
    external_urls: RawExternalUrls,
    total_tracks: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawTrackArtist {
    external_urls: Value,
    href: String,
    id: String,
    name: String,
    r#type: String,
    uri: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawImage {
    url: String,
    width: Option<usize>,
    height: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawTrackExternalIds {
    isrc: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RawExternalUrls {
    spotify: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SessionInfo {
    pub access_token: String,
    pub access_token_expiration_timestamp_ms: usize,
    pub is_anonymous: bool,
    pub client_id: String,
}

impl Spotify {
    const API_BASE_URL: &'static str = "https://api.spotify.com/v1";
    const SITE_BASE_URL: &'static str = "https://open.spotify.com";

    pub async fn get_public_session_info(client: &Client) -> Result<SessionInfo> {
        let request: RequestBuilder = client.get(Self::SITE_BASE_URL);
        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let raw_html: String = response.text().await?;
        let re = regex::Regex::new(r#"(\{"accessToken":.*"\})"#)?;

        let raw_session_info: regex::Match = match re.captures(&raw_html) {
            Some(captures) => match captures.get(1) {
                Some(m) => m,
                None => return Err(Error::SessionGrabError),
            },
            None => return Err(Error::SessionGrabError),
        };

        let session_info: SessionInfo = serde_json::from_str(raw_session_info.as_str())?;

        Ok(session_info)
    }

    async fn get(client: &Client, auth: &str, path: &str) -> Result<Value> {
        let request: RequestBuilder = client
            .get(format!("{}/{}", Self::API_BASE_URL, path))
            .header("Authorization", format!("Bearer {}", auth));
        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let data: Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    pub async fn get_raw_track_match_from_track(
        client: &Client,
        auth: &str,
        track: &Track,
    ) -> Result<RawTrack> {
        match &track.isrc {
            Some(isrc) => {
                match Self::get(
                    client,
                    auth,
                    &format!("search?type=track&q=isrc:{}%20album:{}", isrc, &track.album),
                )
                .await
                {
                    Ok(mut raw_result) => {
                        if raw_result["tracks"]["items"]
                            .as_array()
                            .ok_or(Error::MatchError)?
                            .len()
                            > 0
                        {
                            return Ok(serde_json::from_value(
                                raw_result["tracks"]["items"]
                                    .get_mut(0)
                                    .ok_or(Error::CreateError)?
                                    .take(),
                            )?);
                        }
                    }
                    Err(..) => (),
                }
            }
            None => (),
        }
        // no isrc or isrc search failed

        Ok(serde_json::from_value::<RawTrack>(
            Self::get(
                client,
                auth,
                &format!(
                    "search?type=track&q=track:{}%20artist:{}%20album:{}%20year:{}",
                    track.name,
                    track.artists.join("+"),
                    track.album,
                    track.release_year
                )
                .replace(" ", "+"),
            )
            .await?["tracks"]["items"]
                .get_mut(0)
                .ok_or(Error::CreateError)?
                .take(),
        )?)
    }

    pub async fn create_service_for_track(
        client: &Client,
        auth: &str,
        track: &mut Track,
    ) -> Result<()> {
        let data: RawTrack = Self::get_raw_track_match_from_track(client, auth, track).await?;
        let service: Self = Self::create_service_from_raw(&data).await?;
        track.services.spotify = Some(service);
        Ok(())
    }

    pub async fn create_track_from_id(
        client: &Client,
        auth: &str,
        track_id: &str,
    ) -> Result<Track> {
        let track_data: RawTrack = serde_json::from_value(
            Self::get(client, auth, &format!("tracks/{}", track_id)).await?,
        )?;

        Self::create_track_from_raw(&track_data).await
    }

    pub async fn create_playlist_from_id(
        client: &Client,
        auth: &str,
        playlist_id: &str,
    ) -> Result<Playlist> {
        let playlist_data: RawPlaylist = serde_json::from_value(
            Self::get(client, auth, &format!("playlists/{}", playlist_id)).await?,
        )?;

        let mut tracks_raw: Vec<RawTrack> = Vec::with_capacity(playlist_data.tracks.total);

        if playlist_data.tracks.next == None {
            for track in playlist_data.tracks.items {
                tracks_raw.push(track.track)
            }
        } else {
            eprintln!(
                "There are more than 100 items in playlist, and I haven't implemented pagination"
            );
            return Err(Error::CreateError);
        }

        Self::create_playlist_from_raw(&tracks_raw).await
    }

    pub async fn create_service_from_raw(raw_track: &RawTrack) -> Result<Self> {
        let mut artists: Vec<Artist> = Vec::new();
        for artist in &raw_track.artists {
            artists.push(Artist {
                id: artist.id.to_owned(),
                name: artist.name.to_owned(),
                url: artist.uri.to_owned(),
            })
        }

        Ok(Spotify {
            id: raw_track.id.to_owned(),
            name: raw_track.name.to_owned(),
            url: raw_track.external_urls.spotify.to_owned(),
            artists,
            album: Album {
                id: raw_track.album.id.to_owned(),
                name: raw_track.album.name.to_owned(),
                url: raw_track.album.uri.to_owned(),
                total_tracks: Some(raw_track.album.total_tracks),
                ean: None,
                upc: None,
            },
            duration_ms: raw_track.duration_ms,
            image: {
                if raw_track.album.images.len() > 0 {
                    Some(
                        raw_track
                            .album
                            .images
                            .get(0)
                            .ok_or(Error::CreateError)?
                            .url
                            .to_owned(),
                    )
                } else {
                    None
                }
            },
            audio_preview: raw_track.preview_url.to_owned(),
        })
    }

    pub async fn create_track_from_raw(raw_track: &RawTrack) -> Result<Track> {
        let mut artists: Vec<String> = Vec::new();
        for artist in &raw_track.artists {
            artists.push(artist.name.to_owned());
        }

        let mut release_date: std::str::Split<&str> = raw_track.album.release_date.split("-");

        Ok(Track {
            name: raw_track.name.to_owned(),
            album: raw_track.album.name.to_owned(),
            disk_number: raw_track.disc_number,
            track_number: raw_track.track_number,
            artists,
            release_year: match release_date.next() {
                Some(year) => year.parse().or(Err(Error::CreateError))?,
                None => return Err(Error::CreateError),
            },
            release_month: match release_date.next() {
                Some(month) => Some(month.parse().or(Err(Error::CreateError))?),
                None => None,
            },
            release_day: match release_date.next() {
                Some(day) => Some(day.parse().or(Err(Error::CreateError))?),
                None => None,
            },
            is_explicit: raw_track.explicit,
            duration_ms: raw_track.duration_ms,
            services: Services {
                spotify: Some(Self::create_service_from_raw(raw_track).await?),
                apple_music: None,
                youtube: None,
                bandcamp: None,
            },
            isrc: raw_track.external_ids.isrc.to_owned(),
        })
    }

    pub async fn create_playlist_from_raw(raw_tracks: &Vec<RawTrack>) -> Result<Playlist> {
        let mut new_tracks_futures = Vec::new();
        for raw_track in raw_tracks {
            new_tracks_futures.push(Self::create_track_from_raw(&raw_track));
        }

        let new_tracks_results = futures::future::join_all(new_tracks_futures).await;

        let mut new_tracks: Playlist = Vec::new();
        for track_result in new_tracks_results {
            new_tracks.push(track_result?);
        }

        Ok(new_tracks)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        service::Services,
        spotify::{SessionInfo, Spotify},
        track::Track,
    };

    #[tokio::test]
    async fn get_match_with_isrc() {
        let example_services: Services = Services {
            spotify: None,
            apple_music: None,
            youtube: None,
            bandcamp: None,
        };

        let mut example_track: Track = Track {
            name: "Duchess for Nothing".to_owned(),
            album: "Genius Fatigue".to_owned(),
            disk_number: 1,
            track_number: 1,
            artists: vec!["Tunabunny".to_owned()],
            release_year: 2013,
            release_month: None,
            release_day: None,
            is_explicit: false,
            duration_ms: 138026,
            services: example_services,
            isrc: Some("USZUD1215001".to_owned()),
        };

        let client: reqwest::Client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                    .build()
                    .unwrap();
        let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();

        example_track
            .add_spotify(&session_info.access_token, &client)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_match_no_isrc() {
        let example_services: Services = Services {
            spotify: None,
            apple_music: None,
            youtube: None,
            bandcamp: None,
        };

        let mut example_track: Track = Track {
            name: "Duchess for Nothing".to_owned(),
            album: "Genius Fatigue".to_owned(),
            disk_number: 1,
            track_number: 1,
            artists: vec!["Tunabunny".to_owned()],
            release_year: 2013,
            release_month: None,
            release_day: None,
            is_explicit: false,
            duration_ms: 138026,
            services: example_services,
            isrc: None,
        };

        let client: reqwest::Client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                    .build()
                    .unwrap();
        let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();

        example_track
            .add_spotify(&session_info.access_token, &client)
            .await
            .unwrap();
    }
}
