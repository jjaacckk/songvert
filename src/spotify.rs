use crate::error::{Error, Result};
use crate::service::{Album, Artist, Service, Services};
use crate::track::Track;
use regex::Regex;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RawTrack {}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RawPlaylist {}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RawAlbum {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Spotify {
    pub id: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub url: String,
    pub image: Option<String>,
    pub audio_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(deny_unknown_fields, rename_all(deserialize = "snake_case"))]
pub struct SessionInfo {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "accessTokenExpirationTimestampMs")]
    pub access_token_expiration_timestamp_ms: u64,
    #[serde(rename = "isAnonymous")]
    pub is_anonymous: bool,
    #[serde(rename = "clientId")]
    pub client_id: String,
}

impl Spotify {
    pub async fn get_public_session_info(client: &Client) -> Result<SessionInfo> {
        let request: RequestBuilder = client.get("https://open.spotify.com");
        let response: Response = request.send().await?;

        if response.status() != 200 {
            return Err(Error::SessionGrabError);
        }

        let raw_html: String = response.text().await?;

        let re = Regex::new(r#"(\{"accessToken":.*"\})"#)?;
        let captures: regex::Captures = match re.captures(&raw_html) {
            Some(c) => c,
            None => return Err(Error::SessionGrabError),
        };

        let capture: &str = match captures.get(0) {
            Some(c) => c.as_str(),
            None => return Err(Error::SessionGrabError),
        };
        let session_info: SessionInfo = serde_json::from_str(capture)?;

        Ok(session_info)
    }

    pub async fn search_by_isrc(
        client: &Client,
        auth_token: &str,
        isrc: &str,
    ) -> Result<serde_json::Value> {
        let request: RequestBuilder = client
            .get(format!(
                "https://api.spotify.com/v1/search?type=track&q=isrc:{}",
                isrc
            ))
            .header("Authorization", format!("Bearer {}", auth_token));
        let response: Response = request.send().await?;
        if response.status() != 200 {
            // eprintln!("{}", response.text().await?);
            return Err(Error::FindError);
        }

        let search_data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        if search_data["tracks"]["total"]
            .as_u64()
            .ok_or(Error::FindError)?
            == 0
        {
            return Err(Error::FindError);
        }

        Ok(search_data["tracks"]["items"][0].to_owned())
    }

    // fn search_by_name(name: &str) -> serde_json::Value {}
}

impl Service for Spotify {
    async fn get_raw_track_match(
        client: &Client,
        auth_token: &str,
        track: &Track,
    ) -> Result<serde_json::Value> {
        // match track.isrc {
        //     Some(isrc) => data = Self::search_by_isrc(&isrc),
        //     None => {}
        // }

        Ok(serde_json::Value::from(""))
    }

    async fn add_service_to_track(
        client: &Client,
        auth_token: &str,
        track: &mut Track,
    ) -> Result<()> {
        let data: serde_json::Value = Self::get_raw_track_match(client, auth_token, track).await?;
        let service: Spotify = Self::create_service_from_raw(&data).await?;
        track.services.spotify = Some(service);
        Ok(())
    }

    async fn create_service_from_raw(data: &serde_json::Value) -> Result<Spotify> {
        let mut artists: Vec<Artist> = Vec::new();
        for artist in data["artists"].as_array().ok_or(Error::CreateError)? {
            artists.push(Artist {
                id: artist["id"].as_str().ok_or(Error::CreateError)?.to_owned(),
                name: artist["name"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
            })
        }

        Ok(Spotify {
            id: data["id"].as_str().ok_or(Error::CreateError)?.to_owned(),
            artists,
            album: Album {
                id: data["album"]["id"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
                name: data["album"]["name"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
                total_tracks: data["album"]["total_tracks"]
                    .as_u64()
                    .ok_or(Error::CreateError)?
                    .try_into()
                    .or(Err(Error::CreateError))?,
                ean: None,
                upc: None,
            },
            url: data["external_urls"]["spotify"]
                .as_str()
                .ok_or(Error::CreateError)?
                .to_owned(),
            image: match data["album"]["images"][0]["url"].as_str() {
                Some(url) => Some(url.to_owned()),
                None => None,
            },
            audio_preview: match data["preview_url"].as_str() {
                Some(url) => Some(url.to_owned()),
                None => None,
            },
        })
    }

    async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track> {
        let mut artists: Vec<String> = Vec::new();
        for artist in data["artists"].as_array().ok_or(Error::CreateError)? {
            artists.push(
                artist["name"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
            )
        }

        let mut release_date: std::str::Split<&str> = data["album"]["release_date"]
            .as_str()
            .ok_or(Error::CreateError)?
            .split("-");

        Ok(Track {
            title: data["name"].as_str().ok_or(Error::CreateError)?.to_owned(),
            album: data["album"]["name"]
                .as_str()
                .ok_or(Error::CreateError)?
                .to_owned(),
            disk_number: data["disc_number"]
                .as_u64()
                .ok_or(Error::CreateError)?
                .try_into()
                .or(Err(Error::CreateError))?,
            track_number: data["track_number"]
                .as_u64()
                .ok_or(Error::CreateError)?
                .try_into()
                .or(Err(Error::CreateError))?,
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
            is_explicit: data["explicit"].as_bool().ok_or(Error::CreateError)?,
            duration_ms: data["duration_ms"].as_u64().ok_or(Error::CreateError)?,
            services: Services {
                spotify: Some(Spotify::create_service_from_raw(data).await?),
                apple_music: None,
                youtube: None,
                bandcamp: None,
            },
            isrc: match data["external_ids"]["isrc"].as_str() {
                Some(isrc) => Some(isrc.to_owned()),
                None => None,
            },
        })
    }

    async fn create_track_from_id(
        client: &Client,
        auth_token: &str,
        track_id: &str,
    ) -> Result<Track> {
        let request: RequestBuilder = client
            .get(format!("https://api.spotify.com/v1/tracks/{}", track_id))
            .header("Authorization", format!("Bearer {}", auth_token));
        let response: Response = request.send().await?;
        if response.status() != 200 {
            // eprintln!("{}", response.text().await?);
            return Err(Error::FindError);
        }
        let track_data: serde_json::Value = serde_json::from_str(&response.text().await?)?;
        Ok(Self::create_track_from_raw(&track_data).await?)
    }

    async fn create_tracks_from_playlist_raw(data: &serde_json::Value) -> Result<Vec<Track>> {
        let new_tracks: Vec<Track> = Vec::new();
        Ok(new_tracks)
    }

    async fn create_tracks_from_playlist_id(
        client: &Client,
        auth_token: &str,
        playlist_id: &str,
    ) -> Result<Vec<Track>> {
        let request: RequestBuilder = client
            .get(format!(
                "https://api.spotify.com/v1/playlists/{}",
                playlist_id
            ))
            .header("Authorizaiton", format!("Bearer {}", auth_token));
        let response: Response = request.send().await?;
        if response.status() != 200 {
            return Err(Error::FindError);
        }
        let playlist_data: serde_json::Value = serde_json::Value::from(response.text().await?);
        Ok(Self::create_tracks_from_playlist_raw(&playlist_data).await?)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        service::Service,
        spotify::{SessionInfo, Spotify},
        track::{Playlist, Track},
    };

    #[tokio::test]
    async fn good_isrc_search() {
        let good_isrc: &str = "GBAYK8000001";
        let client: reqwest::Client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                .build()
                .unwrap();
        let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();

        let search_result: serde_json::Value =
            Spotify::search_by_isrc(&client, &session_info.access_token, good_isrc)
                .await
                .unwrap();

        assert_eq!(
            search_result,
            serde_json::from_str::<serde_json::Value>(
                r#"{ "album": { "album_type": "album", "artists": [ { "external_urls": { "spotify": "https://open.spotify.com/artist/3wRksusBxJ6npu0PryYheF" }, "href": "https://api.spotify.com/v1/artists/3wRksusBxJ6npu0PryYheF", "id": "3wRksusBxJ6npu0PryYheF", "name": "The Selecter", "type": "artist", "uri": "spotify:artist:3wRksusBxJ6npu0PryYheF" } ], "available_markets": [ "AR", "AU", "AT", "BE", "BO", "BR", "BG", "CA", "CL", "CO", "CR", "CY", "CZ", "DK", "DO", "DE", "EC", "EE", "SV", "FI", "FR", "GR", "GT", "HN", "HK", "HU", "IS", "IE", "IT", "LV", "LT", "LU", "MY", "MT", "MX", "NL", "NZ", "NI", "NO", "PA", "PY", "PE", "PH", "PL", "PT", "SG", "SK", "ES", "SE", "CH", "TW", "TR", "UY", "US", "GB", "AD", "LI", "MC", "ID", "JP", "TH", "VN", "RO", "IL", "ZA", "SA", "AE", "BH", "QA", "OM", "KW", "EG", "MA", "DZ", "TN", "LB", "JO", "PS", "IN", "BY", "KZ", "MD", "UA", "AL", "BA", "HR", "ME", "MK", "RS", "SI", "KR", "BD", "PK", "LK", "GH", "KE", "NG", "TZ", "UG", "AG", "AM", "BS", "BB", "BZ", "BT", "BW", "BF", "CV", "CW", "DM", "FJ", "GM", "GE", "GD", "GW", "GY", "HT", "JM", "KI", "LS", "LR", "MW", "MV", "ML", "MH", "FM", "NA", "NR", "NE", "PW", "PG", "PR", "WS", "SM", "ST", "SN", "SC", "SL", "SB", "KN", "LC", "VC", "SR", "TL", "TO", "TT", "TV", "VU", "AZ", "BN", "BI", "KH", "CM", "TD", "KM", "GQ", "SZ", "GA", "GN", "KG", "LA", "MO", "MR", "MN", "NP", "RW", "TG", "UZ", "ZW", "BJ", "MG", "MU", "MZ", "AO", "CI", "DJ", "ZM", "CD", "CG", "IQ", "LY", "TJ", "VE", "ET", "XK" ], "external_urls": { "spotify": "https://open.spotify.com/album/5wnNTpK8zZCzbEblOvKmUV" }, "href": "https://api.spotify.com/v1/albums/5wnNTpK8zZCzbEblOvKmUV", "id": "5wnNTpK8zZCzbEblOvKmUV", "images": [ { "height": 640, "url": "https://i.scdn.co/image/ab67616d0000b273d0578b6c0b35405fc1f66b99", "width": 640 }, { "height": 300, "url": "https://i.scdn.co/image/ab67616d00001e02d0578b6c0b35405fc1f66b99", "width": 300 }, { "height": 64, "url": "https://i.scdn.co/image/ab67616d00004851d0578b6c0b35405fc1f66b99", "width": 64 } ], "name": "Too Much Pressure", "release_date": "1980-02-23", "release_date_precision": "day", "total_tracks": 13, "type": "album", "uri": "spotify:album:5wnNTpK8zZCzbEblOvKmUV" }, "artists": [ { "external_urls": { "spotify": "https://open.spotify.com/artist/3wRksusBxJ6npu0PryYheF" }, "href": "https://api.spotify.com/v1/artists/3wRksusBxJ6npu0PryYheF", "id": "3wRksusBxJ6npu0PryYheF", "name": "The Selecter", "type": "artist", "uri": "spotify:artist:3wRksusBxJ6npu0PryYheF" } ], "available_markets": [ "AR", "AU", "AT", "BE", "BO", "BR", "BG", "CA", "CL", "CO", "CR", "CY", "CZ", "DK", "DO", "DE", "EC", "EE", "SV", "FI", "FR", "GR", "GT", "HN", "HK", "HU", "IS", "IE", "IT", "LV", "LT", "LU", "MY", "MT", "MX", "NL", "NZ", "NI", "NO", "PA", "PY", "PE", "PH", "PL", "PT", "SG", "SK", "ES", "SE", "CH", "TW", "TR", "UY", "US", "GB", "AD", "LI", "MC", "ID", "JP", "TH", "VN", "RO", "IL", "ZA", "SA", "AE", "BH", "QA", "OM", "KW", "EG", "MA", "DZ", "TN", "LB", "JO", "PS", "IN", "BY", "KZ", "MD", "UA", "AL", "BA", "HR", "ME", "MK", "RS", "SI", "KR", "BD", "PK", "LK", "GH", "KE", "NG", "TZ", "UG", "AG", "AM", "BS", "BB", "BZ", "BT", "BW", "BF", "CV", "CW", "DM", "FJ", "GM", "GE", "GD", "GW", "GY", "HT", "JM", "KI", "LS", "LR", "MW", "MV", "ML", "MH", "FM", "NA", "NR", "NE", "PW", "PG", "PR", "WS", "SM", "ST", "SN", "SC", "SL", "SB", "KN", "LC", "VC", "SR", "TL", "TO", "TT", "TV", "VU", "AZ", "BN", "BI", "KH", "CM", "TD", "KM", "GQ", "SZ", "GA", "GN", "KG", "LA", "MO", "MR", "MN", "NP", "RW", "TG", "UZ", "ZW", "BJ", "MG", "MU", "MZ", "AO", "CI", "DJ", "ZM", "CD", "CG", "IQ", "LY", "TJ", "VE", "ET", "XK" ], "disc_number": 1, "duration_ms": 183273, "explicit": false, "external_ids": { "isrc": "GBAYK8000001" }, "external_urls": { "spotify": "https://open.spotify.com/track/3iWEwCUX3OrJkHmadDCNgC" }, "href": "https://api.spotify.com/v1/tracks/3iWEwCUX3OrJkHmadDCNgC", "id": "3iWEwCUX3OrJkHmadDCNgC", "is_local": false, "name": "Three Minute Hero", "popularity": 37, "preview_url": "https://p.scdn.co/mp3-preview/0c1eab05a18935fedba5c3a343b5de2dce5d4bbb?cid=d8a5ed958d274c2e8ee717e6a4b0971d", "track_number": 1, "type": "track", "uri": "spotify:track:3iWEwCUX3OrJkHmadDCNgC" }"#
            ).unwrap()
        );
    }

    #[tokio::test]
    async fn bad_isrc_search() {
        let bad_isrc: &str = "FAKE_ISRC";
        let client: reqwest::Client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                .build()
                .unwrap();
        let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();

        match Spotify::search_by_isrc(&client, &session_info.access_token, bad_isrc).await {
            Ok(..) => panic!(),
            Err(..) => {}
        }
    }

    // #[tokio::test]
    // async fn get_playlist() {
    //     let playlist_id: &str = "3Js7lLCb0uIyodaTmjTSWv";
    //     let client: reqwest::Client = reqwest::Client::builder()
    //         .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
    //         .build()
    //         .unwrap();
    //     let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();
    //     let playlist: Playlist = Spotify::create_tracks_from_playlist_id(
    //         &client,
    //         &session_info.access_token,
    //         playlist_id,
    //     )
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // async fn get_track() {
    //     let track_id: &str = "2egOhUwX8qqjUj0QzLkLdO";
    //     let client: reqwest::Client = reqwest::Client::builder()
    //             .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
    //             .build()
    //             .unwrap();
    //     let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();
    //     let track: Track =
    //         Spotify::create_track_from_id(&client, &session_info.access_token, track_id)
    //             .await
    //             .unwrap();
    // }
}
