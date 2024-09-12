use crate::error::{Error, Result};
use crate::service::{Album, Artist, Services};
use crate::track::{Playlist, Track};
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AppleMusic {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub composer: Option<String>,
    pub album: Album,
    pub duration_ms: usize,
    pub image: Option<String>,
    pub genres: Vec<String>,
    pub audio_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawPlaylist {
    pub id: String,
    pub r#type: String,
    pub href: String,
    pub attributes: Option<RawPlaylistAttributes>,
    pub relationships: Option<RawPlaylistRelationships>,
    pub views: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawPlaylistAttributes {
    pub artwork: RawArtwork,
    pub curator_name: String,
    pub description: Option<Value>,
    pub is_chart: bool,
    pub last_modified_date: Option<String>,
    pub name: String,
    pub playlist_type: String,
    pub play_params: Option<Value>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawPlaylistRelationships {
    pub curator: Option<Value>,
    pub library: Option<Value>,
    pub tracks: Option<RawTracks>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawTracks {
    pub data: Vec<RawTracks>,
    pub href: Option<String>,
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawTrack {
    pub id: String,
    pub r#type: String,
    pub href: String,
    pub attributes: Option<RawTrackAttributes>,
    pub relationships: Option<RawTrackRelationships>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawTrackRelationships {
    pub albums: Option<RawAlbums>,
    pub artists: Option<RawArtists>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawAlbums {
    pub data: Vec<RawAlbum>,
    pub href: Option<String>,
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawAlbum {
    pub id: String,
    pub href: String,
    pub r#type: String,
    pub attributes: Option<RawAlbumAttributes>,
    pub relationships: Option<Value>,
    pub views: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawAlbumAttributes {
    pub artist_name: String,
    pub artwork: RawArtwork,
    pub content_rating: Option<String>,
    pub copyright: Option<String>,
    pub editorial_notes: Option<Value>,
    pub genre_names: Vec<String>,
    pub is_compilation: bool,
    pub is_complete: bool,
    pub is_mastered_for_itunes: bool,
    pub is_single: bool,
    pub name: String,
    pub play_params: Option<Value>,
    pub record_label: Option<String>,
    pub release_date: Option<String>,
    pub track_count: usize,
    pub upc: Option<String>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawArtists {
    pub data: Vec<RawArtist>,
    pub href: Option<String>,
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawArtist {
    pub id: String,
    pub href: String,
    pub r#type: String,
    pub attributes: Option<RawArtistAttributes>,
    pub relationships: Option<RawArtistRelationships>,
    pub views: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawArtistAttributes {
    pub artwork: Option<RawArtwork>,
    pub editorial_notes: Option<Value>,
    pub genre_names: Vec<String>,
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawArtistRelationships {
    pub albums: Option<RawAlbums>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawTrackAttributes {
    pub album_name: String,
    pub artist_name: String,
    pub artwork: RawArtwork,
    pub attribution: Option<String>,
    pub composer_name: Option<String>,
    pub content_rating: Option<String>,
    pub disc_number: Option<usize>,
    pub duration_in_millis: usize,
    pub genre_names: Vec<String>,
    pub has_lyrics: bool,
    pub is_apple_digital_master: bool,
    pub isrc: Option<String>,
    pub name: String,
    pub play_params: Option<Value>,
    pub previews: Vec<RawTrackAttributesPreview>,
    pub release_date: Option<String>,
    pub track_number: Option<usize>,
    pub url: String,
    pub work_name: Option<String>,
    pub movement_count: Option<String>,
    pub movement_name: Option<String>,
    pub movement_number: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawTrackAttributesPreview {
    pub url: String,
    pub artwork: Option<RawArtwork>,
    pub hls_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawArtwork {
    pub bg_color: Option<String>,
    pub height: usize,
    pub width: usize,
    pub text_color1: Option<String>,
    pub text_color2: Option<String>,
    pub text_color3: Option<String>,
    pub text_color4: Option<String>,
    pub url: String,
}

impl AppleMusic {
    pub const API_BASE_URL: &'static str = "https://api.music.apple.com/v1";
    pub const SITE_BASE_URL: &'static str = "https://music.apple.com";

    pub const PUBLIC_BEARER_TOKEN: &'static str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldlYlBsYXlLaWQifQ.eyJpc3MiOiJBTVBXZWJQbGF5IiwiaWF0IjoxNzI0MDk1NzA5LCJleHAiOjE3MzEzNTMzMDksInJvb3RfaHR0cHNfb3JpZ2luIjpbImFwcGxlLmNvbSJdfQ.upq7QFkHN3etQVMycsuIlNYz8ElqqZeNCZ2OJUTHie2IbRiExsZJejLdQgv0JysNSu-w63IcOW6GCVbImMV3Zw";

    pub async fn get(client: &Client, auth: &str, path: &str) -> Result<serde_json::Value> {
        let request: RequestBuilder = client
            .get(format!("{}/{}", Self::API_BASE_URL, path))
            .header("Authorization", format!("Bearer {}", auth))
            .header("Origin", Self::SITE_BASE_URL);
        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    pub async fn get_raw_track_matches_from_tracks(
        client: &Client,
        auth: &str,
        tracks: &Vec<Track>,
    ) -> Result<Vec<RawTracks>> {
        let isrcs: Vec<Option<&str>> = Vec::new();
        todo!()
    }

    pub async fn get_raw_track_match_from_track(
        client: &Client,
        auth: &str,
        track: &Track,
    ) -> Result<RawTrack> {
        match &track.isrc {
            Some(isrc) => match Self::get(
                client,
                auth,
                &format!(
                    "catalog/us/songs?filter[isrc]={}&include=albums,artists",
                    isrc
                ),
            )
            .await
            {
                Ok(mut raw_data) => {
                    let mut raw_tracks: Vec<RawTrack> =
                        serde_json::from_value(raw_data["data"].take())?;
                    if raw_tracks.len() > 1 {
                        // check album name
                        for i in 0..raw_tracks.len() {
                            if raw_tracks[i]
                                .attributes
                                .as_ref()
                                .ok_or(Error::MatchError)?
                                .album_name
                                .to_lowercase()
                                == track.album.to_lowercase()
                            {
                                return Ok(raw_tracks.remove(i));
                            }
                        }
                    }

                    // only one result or album name can't be found
                    if raw_tracks.len() != 0 {
                        return Ok(raw_tracks.remove(0));
                    }
                }
                Err(..) => (),
            },
            None => (),
        }
        // no isrc or isrc search failed
        eprintln!("isrc search failed for {}....fallback....", track.name);

        let mut lackluster_search_result: serde_json::Value = Self::get(
            client,
            auth,
            &format!(
                "catalog/us/search?types=songs&term=song:{}%20artist:{}%20album:{}%20year:{}",
                &track.name,
                &track.artists.get(0).ok_or(Error::MalformedTrackError)?,
                &track.album,
                &track.release_year
            )
            .replace(" ", "+")
            .replace(&['\'', ','], ""),
        )
        .await?;

        let lsr_raw_tracks: Vec<RawTrack> =
            serde_json::from_value(lackluster_search_result["results"]["songs"]["data"].take())?;

        for i in 0..lsr_raw_tracks.len() {
            if track.compare_similarity(
                &lsr_raw_tracks[i]
                    .attributes
                    .as_ref()
                    .ok_or(Error::MatchError)?
                    .name,
                &lsr_raw_tracks[i]
                    .attributes
                    .as_ref()
                    .ok_or(Error::MatchError)?
                    .artist_name,
                &lsr_raw_tracks[i]
                    .attributes
                    .as_ref()
                    .ok_or(Error::MatchError)?
                    .album_name,
                lsr_raw_tracks[i]
                    .attributes
                    .as_ref()
                    .ok_or(Error::MatchError)?
                    .duration_in_millis,
            ) >= 3
            {
                match Self::get(
                    client,
                    auth,
                    &format!(
                        "catalog/us/songs/{}?include=artists,albums",
                        lsr_raw_tracks[i].id
                    ),
                )
                .await
                {
                    Ok(mut data) => {
                        return Ok(serde_json::from_value(
                            data["data"].get_mut(0).ok_or(Error::MatchError)?.take(),
                        )?)
                    }
                    Err(e) => return Err(e),
                };
            }
        }

        Err(Error::MatchError)
    }

    pub async fn create_service_for_track(
        client: &Client,
        auth: &str,
        track: &mut Track,
    ) -> Result<()> {
        let data: RawTrack = Self::get_raw_track_match_from_track(client, auth, track).await?;
        let service: Self = Self::create_service_from_raw(&data).await?;
        track.services.apple_music = Some(service);
        Ok(())
    }

    pub async fn create_track_from_id(
        client: &Client,
        auth: &str,
        track_id: &str,
    ) -> Result<Track> {
        match Self::get(
            client,
            auth,
            &format!("catalog/us/songs/{}?include=artists,albums", track_id),
        )
        .await
        {
            Ok(mut track_data) => Ok(Self::create_track_from_raw(&serde_json::from_value(
                track_data["data"]
                    .get_mut(0)
                    .ok_or(Error::CreateError)?
                    .take(),
            )?)
            .await?),
            Err(e) => Err(e),
        }
    }

    pub async fn create_playlist_from_id(
        client: &Client,
        auth: &str,
        playlist_id: &str,
    ) -> Result<Playlist> {
        todo!()
    }

    pub async fn create_service_from_raw(raw_track: &RawTrack) -> Result<Self> {
        let relationships: &RawTrackRelationships =
            raw_track.relationships.as_ref().ok_or(Error::CreateError)?;
        let albums: &RawAlbums = relationships.albums.as_ref().ok_or(Error::CreateError)?;
        let first_album_attributes: &RawAlbumAttributes = albums
            .data
            .get(0)
            .ok_or(Error::CreateError)?
            .attributes
            .as_ref()
            .ok_or(Error::CreateError)?;
        let attributes: &RawTrackAttributes =
            raw_track.attributes.as_ref().ok_or(Error::CreateError)?;

        let mut artists: Vec<Artist> = Vec::new();
        for artist in &relationships
            .artists
            .as_ref()
            .ok_or(Error::CreateError)?
            .data
        {
            let artist_attributes = artist.attributes.as_ref().ok_or(Error::CreateError)?;
            artists.push(Artist {
                id: artist.id.to_owned(),
                name: artist_attributes.name.to_owned(),
                url: artist_attributes.url.to_owned(),
            })
        }

        Ok(AppleMusic {
            id: raw_track.id.to_owned(),
            name: attributes.name.to_owned(),
            url: attributes.url.to_owned(),
            artists,
            album: Album {
                id: albums.data.get(0).ok_or(Error::CreateError)?.id.to_owned(),
                name: first_album_attributes.name.to_owned(),
                url: first_album_attributes.url.to_owned(),
                total_tracks: Some(first_album_attributes.track_count),
                ean: None,
                upc: first_album_attributes.upc.to_owned(),
            },
            duration_ms: attributes.duration_in_millis,
            image: Some(
                attributes
                    .artwork
                    .url
                    .replace("{w}x{h}bb.jpg", "352x352bb.webp"),
            ),
            audio_preview: {
                if attributes.previews.len() > 0 {
                    Some(
                        attributes
                            .previews
                            .get(0)
                            .ok_or(Error::CreateError)?
                            .url
                            .to_owned(),
                    )
                } else {
                    None
                }
            },
            genres: attributes.genre_names.to_owned(),
            composer: match &attributes.composer_name {
                Some(composer) => Some(composer.to_owned()),
                None => None,
            },
        })
    }

    pub async fn create_track_from_raw(raw_track: &RawTrack) -> Result<Track> {
        let relationships: &RawTrackRelationships =
            raw_track.relationships.as_ref().ok_or(Error::CreateError)?;
        let albums: &RawAlbums = relationships.albums.as_ref().ok_or(Error::CreateError)?;
        let first_album_attributes: &RawAlbumAttributes = albums
            .data
            .get(0)
            .ok_or(Error::CreateError)?
            .attributes
            .as_ref()
            .ok_or(Error::CreateError)?;
        let attributes: &RawTrackAttributes =
            raw_track.attributes.as_ref().ok_or(Error::CreateError)?;

        let mut artists: Vec<String> = Vec::new();
        for artist in &relationships
            .artists
            .as_ref()
            .ok_or(Error::CreateError)?
            .data
        {
            artists.push(
                artist
                    .attributes
                    .as_ref()
                    .ok_or(Error::CreateError)?
                    .name
                    .to_owned(),
            )
        }

        let mut release_date: std::str::Split<&str> = attributes
            .release_date
            .as_ref()
            .ok_or(Error::CreateError)?
            .split("-");

        Ok(Track {
            name: attributes.name.to_owned(),
            album: attributes.album_name.to_owned(),
            disk_number: attributes.disc_number.ok_or(Error::CreateError)?,
            track_number: attributes.track_number.ok_or(Error::CreateError)?,
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
            is_explicit: match &attributes.content_rating {
                Some(content_rating) => {
                    if content_rating == "explicit" {
                        true
                    } else {
                        false
                    }
                }
                None => false,
            },
            duration_ms: attributes.duration_in_millis,
            services: Services {
                spotify: None,
                apple_music: Some(Self::create_service_from_raw(raw_track).await?),
                youtube: None,
                bandcamp: None,
            },
            isrc: match &attributes.isrc {
                Some(isrc) => Some(isrc.to_owned()),
                None => None,
            },
        })
    }

    pub async fn create_playlist_from_raw(raw_tracks: &Vec<RawTrack>) -> Result<Playlist> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::{apple_music::AppleMusic, service::Services, track::Track};

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

        example_track
            .add_apple_music(AppleMusic::PUBLIC_BEARER_TOKEN, &client)
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

        example_track
            .add_apple_music(AppleMusic::PUBLIC_BEARER_TOKEN, &client)
            .await
            .unwrap();
    }
}
