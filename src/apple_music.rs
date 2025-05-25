use crate::error::{Error, Result};
use crate::playlist::Playlist;
use crate::service::{Album, Artist, Services, Source};
use crate::track::Track;
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
    pub image_no_suffix: Option<String>,
    pub genres: Vec<String>,
    pub audio_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawPlaylist {
    id: String,
    r#type: String,
    href: String,
    attributes: Option<RawPlaylistAttributes>,
    relationships: Option<RawPlaylistRelationships>,
    views: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawPlaylistAttributes {
    artwork: RawArtwork,
    curator_name: String,
    description: Option<Value>,
    is_chart: bool,
    last_modified_date: Option<String>,
    name: String,
    playlist_type: String,
    play_params: Option<Value>,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawPlaylistRelationships {
    curator: Option<Value>,
    library: Option<Value>,
    tracks: Option<RawTracks>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawTracks {
    data: Vec<RawTracks>,
    href: Option<String>,
    next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawTrack {
    id: String,
    r#type: String,
    href: String,
    attributes: Option<RawTrackAttributes>,
    relationships: Option<RawTrackRelationships>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawTrackRelationships {
    albums: Option<RawAlbums>,
    artists: Option<RawArtists>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawAlbums {
    data: Vec<RawAlbum>,
    href: Option<String>,
    next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawAlbum {
    id: String,
    href: String,
    r#type: String,
    attributes: Option<RawAlbumAttributes>,
    relationships: Option<Value>,
    views: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawAlbumAttributes {
    artist_name: String,
    artwork: RawArtwork,
    content_rating: Option<String>,
    copyright: Option<String>,
    editorial_notes: Option<Value>,
    genre_names: Vec<String>,
    is_compilation: bool,
    is_complete: bool,
    is_mastered_for_itunes: bool,
    is_single: bool,
    name: String,
    play_params: Option<Value>,
    record_label: Option<String>,
    release_date: Option<String>,
    track_count: usize,
    upc: Option<String>,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawArtists {
    data: Vec<RawArtist>,
    href: Option<String>,
    next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawArtist {
    id: String,
    href: String,
    r#type: String,
    attributes: Option<RawArtistAttributes>,
    relationships: Option<RawArtistRelationships>,
    views: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawArtistAttributes {
    artwork: Option<RawArtwork>,
    editorial_notes: Option<Value>,
    genre_names: Vec<String>,
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawArtistRelationships {
    albums: Option<RawAlbums>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawTrackAttributes {
    album_name: String,
    artist_name: String,
    artwork: RawArtwork,
    attribution: Option<String>,
    composer_name: Option<String>,
    content_rating: Option<String>,
    disc_number: Option<usize>,
    duration_in_millis: usize,
    genre_names: Vec<String>,
    has_lyrics: bool,
    is_apple_digital_master: bool,
    isrc: Option<String>,
    name: String,
    play_params: Option<Value>,
    previews: Vec<RawTrackAttributesPreview>,
    release_date: Option<String>,
    track_number: Option<usize>,
    url: String,
    work_name: Option<String>,
    movement_count: Option<String>,
    movement_name: Option<String>,
    movement_number: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawTrackAttributesPreview {
    url: String,
    artwork: Option<RawArtwork>,
    hls_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct RawArtwork {
    bg_color: Option<String>,
    height: usize,
    width: usize,
    text_color1: Option<String>,
    text_color2: Option<String>,
    text_color3: Option<String>,
    text_color4: Option<String>,
    url: String,
}

impl AppleMusic {
    pub const API_BASE_URL: &'static str = "https://api.music.apple.com/v1";
    pub const SITE_BASE_URL: &'static str = "https://music.apple.com";

    pub const PUBLIC_BEARER_TOKEN: &'static str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldlYlBsYXlLaWQifQ.eyJpc3MiOiJBTVBXZWJQbGF5IiwiaWF0IjoxNzQ0ODMwNzIxLCJleHAiOjE3NTIwODgzMjEsInJvb3RfaHR0cHNfb3JpZ2luIjpbImFwcGxlLmNvbSJdfQ.cIz-EyZfbwgOzioztmbdpgrDwsaYJpDQqYvLP4K4dOF_0zKhCCRQHS_s6VmLJXEa9fxu8-0ScHkOAqxddCvG7Q";

    async fn get(client: &Client, auth: &str, path: &str) -> Result<Value> {
        let request: RequestBuilder = client
            .get(format!("{}/{}", Self::API_BASE_URL, path))
            .header("Authorization", format!("Bearer {}", auth))
            .header("Origin", Self::SITE_BASE_URL);
        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    async fn get_raw_track_matches_from_tracks(
        client: &Client,
        auth: &str,
        tracks: &Vec<Track>,
    ) -> Result<Vec<RawTracks>> {
        let isrcs: Vec<Option<&str>> = Vec::new();
        todo!()
    }

    async fn get_raw_track_match_from_track(
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
                    // check album name
                    for i in 0..raw_tracks.len() {
                        if let Some(attributes) = &raw_tracks[i].attributes {
                            if attributes.album_name.to_lowercase() == track.album.to_lowercase() {
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
        log::info!(
            "ISRC search failed for {}. Using fallback method.",
            track.name
        );

        let mut lackluster_search_result: serde_json::Value = Self::get(
            client,
            auth,
            &format!(
                "catalog/us/search?types=songs&term=song:{}%20artist:{}%20album:{}%20year:{}",
                &track.name,
                &track.artists.get(0).ok_or(Error::TrackError(
                    "Track requires at least one artist".to_string()
                ))?,
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
            if let Some(attributes) = &lsr_raw_tracks[i].attributes {
                if track.compare_similarity_fuzzy(
                    &attributes.name,
                    &attributes.artist_name,
                    &attributes.album_name,
                    attributes.duration_in_millis,
                ) >= 3.0
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
                            if let Some(t) = data["data"].get_mut(0) {
                                return Ok(serde_json::from_value(t.take())?);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Err(Error::TrackError("no match found".to_string()))
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
                    .ok_or(Error::TrackError(format!(
                        "unable to create track from id: {}",
                        track_id
                    )))?
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

    async fn create_service_from_raw(raw_track: &RawTrack) -> Result<Self> {
        let relationships: &RawTrackRelationships = raw_track
            .relationships
            .as_ref()
            .ok_or(Error::DatabaseError("no track relationships".to_string()))?;
        let albums: &RawAlbums = relationships
            .albums
            .as_ref()
            .ok_or(Error::DatabaseError("no albums".to_string()))?;
        let first_album_attributes: &RawAlbumAttributes = albums
            .data
            .get(0)
            .ok_or(Error::DatabaseError("no album".to_string()))?
            .attributes
            .as_ref()
            .ok_or(Error::DatabaseError("no album attributes".to_string()))?;
        let attributes: &RawTrackAttributes = raw_track
            .attributes
            .as_ref()
            .ok_or(Error::DatabaseError("no track attributes".to_string()))?;

        let mut artists: Vec<Artist> = Vec::new();
        for artist in &relationships
            .artists
            .as_ref()
            .ok_or(Error::DatabaseError("no artist relationships".to_string()))?
            .data
        {
            let artist_attributes = artist
                .attributes
                .as_ref()
                .ok_or(Error::DatabaseError("no artist attributes".to_string()))?;
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
                id: albums
                    .data
                    .get(0)
                    .ok_or(Error::DatabaseError("no album".to_string()))?
                    .id
                    .to_owned(),
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
            image_no_suffix: Some(attributes.artwork.url.replace("{w}x{h}bb.jpg", "")),
            audio_preview: {
                if attributes.previews.len() > 0 {
                    Some(attributes.previews[0].url.to_owned())
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

    async fn create_track_from_raw(raw_track: &RawTrack) -> Result<Track> {
        let relationships: &RawTrackRelationships = raw_track
            .relationships
            .as_ref()
            .ok_or(Error::DatabaseError("no track relationships".to_string()))?;
        let attributes: &RawTrackAttributes = raw_track
            .attributes
            .as_ref()
            .ok_or(Error::DatabaseError("no track attributes".to_string()))?;

        let mut artists: Vec<String> = Vec::new();
        for artist in &relationships
            .artists
            .as_ref()
            .ok_or(Error::DatabaseError("no artists".to_string()))?
            .data
        {
            artists.push(
                artist
                    .attributes
                    .as_ref()
                    .ok_or(Error::DatabaseError("no artist attributes".to_string()))?
                    .name
                    .to_owned(),
            )
        }

        let mut release_date: std::str::Split<&str> = attributes
            .release_date
            .as_ref()
            .ok_or(Error::DatabaseError("no release date".to_string()))?
            .split("-");

        Ok(Track {
            name: attributes.name.to_owned(),
            album: attributes.album_name.to_owned(),
            disk_number: attributes
                .disc_number
                .ok_or(Error::DatabaseError("no disc number".to_string()))?,
            track_number: attributes
                .track_number
                .ok_or(Error::DatabaseError("no track number".to_string()))?,
            artists,
            release_year: match release_date.next() {
                Some(year) => year.parse()?,
                None => return Err(Error::DatabaseError("no release year".to_string())),
            },
            release_month: match release_date.next() {
                Some(month) => Some(month.parse()?),
                None => None,
            },
            release_day: match release_date.next() {
                Some(day) => Some(day.parse()?),
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
            source_service: Source::AppleMusic,
        })
    }

    async fn create_playlist_from_raw(raw_tracks: &Vec<RawTrack>) -> Result<Playlist> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        apple_music::AppleMusic,
        service::{Services, Source},
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
            source_service: Source::AppleMusic,
        };

        let client: reqwest::Client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                    .build()
                    .unwrap();

        example_track
            .add_apple_music(&client, AppleMusic::PUBLIC_BEARER_TOKEN)
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
            source_service: Source::AppleMusic,
        };

        let client: reqwest::Client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                    .build()
                    .unwrap();

        example_track
            .add_apple_music(&client, AppleMusic::PUBLIC_BEARER_TOKEN)
            .await
            .unwrap();
    }
}
