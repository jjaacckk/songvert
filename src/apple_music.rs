use crate::error::{Error, Result};
use crate::service::{Album, Artist, Service, Services};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AppleMusic {
    pub id: String,
    pub artists: Vec<Artist>,
    pub composer: Option<String>,
    pub album: Album,
    pub url: String,
    pub image: Option<String>,
    pub genres: Vec<String>,
    pub audio_preview: Option<String>,
}

impl AppleMusic {
    pub const PUBLIC_BEARER_TOKEN: &str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldlYlBsYXlLaWQifQ.eyJpc3MiOiJBTVBXZWJQbGF5IiwiaWF0IjoxNzIxNzczNjI0LCJleHAiOjE3MjkwMzEyMjQsInJvb3RfaHR0cHNfb3JpZ2luIjpbImFwcGxlLmNvbSJdfQ.cMMhHLLazlxgiIbwBSSP1YuHCgqAVxiF7UQrwBc5xZepWt-vjqth_o4BidXFrmsEJvwzZKJ-GAMbqJpIeGcl7w";
}

impl Service for AppleMusic {
    const API_BASE_URL: &'static str = "https://api.music.apple.com/v1";
    // const API_BASE_URL: &'static str = "https://amp-api-edge.music.apple.com/v1";
    const SITE_BASE_URL: &'static str = "https://music.apple.com";

    async fn get_raw(client: &Client, auth_token: &str, path: &str) -> Result<serde_json::Value> {
        let request: RequestBuilder = client
            .get(format!("{}/{}", Self::API_BASE_URL, path))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Origin", AppleMusic::SITE_BASE_URL);
        let response: Response = request.send().await?;
        if response.status() != 200 {
            eprintln!("{}", response.text().await?);
            return Err(Error::FindError);
        }

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    // async fn get_raw_track_match_from_search(
    //     client: &Client,
    //     auth_token: &str,
    //     query: &str,
    // ) -> Result<serde_json::Value> {
    //     let request: RequestBuilder = client
    //         .get(format!(
    //             "{}/catalog/us/search?term={}&type=songs",
    //             AppleMusic::API_BASE_URL,
    //             query
    //         ))
    //         .header("Authorization", format!("Bearer {}", auth_token))
    //         .header("Origin", AppleMusic::SITE_BASE_URL);
    //     let response: Response = request.send().await?;
    //     if response.status() != 200 {
    //         // eprintln!("{}", response.text().await?);
    //         return Err(Error::FindError);
    //     }

    //     let search_data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

    //     if search_data["meta"]["results"]["order"]
    //         .as_array()
    //         .ok_or(Error::FindError)?
    //         .len()
    //         == 0
    //     {
    //         return Err(Error::FindError);
    //     }

    //     Ok(search_data["results"]["songs"]["data"][0].to_owned())
    // }

    async fn get_raw_track_match_from_track(
        client: &Client,
        auth_token: &str,
        track: &Track,
    ) -> Result<serde_json::Value> {
        match &track.isrc {
            Some(isrc) => match Self::get_raw(
                &client,
                &auth_token,
                &format!("catalog/us/songs?filter[isrc]={}", isrc),
            )
            .await
            {
                Ok(raw_track) => return Ok(raw_track),
                Err(..) => (),
            },
            None => (),
        }
        // no isrc or isrc search failed

        Self::get_raw(
            &client,
            &auth_token,
            &format!(
                "catalog/us/search?types=songs&term=track:{}%20artist:{}%20album:{}%20year:{}",
                &track.name, &track.artists[0], &track.album, &track.release_year
            )
            .replace(" ", "+"),
        )
        .await
    }

    async fn create_service_for_track(
        client: &Client,
        auth_token: &str,
        track: &mut Track,
    ) -> Result<()> {
        let data: serde_json::Value =
            Self::get_raw_track_match_from_track(client, auth_token, track).await?;
        let service: Self = Self::create_service_from_raw(&data).await?;
        track.services.apple_music = Some(service);
        Ok(())
    }

    async fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
    where
        Self: Sized,
    {
        let mut artists: Vec<Artist> = Vec::new();
        for artist in data["relationships"]["artists"]["data"]
            .as_array()
            .ok_or(Error::CreateError)?
        {
            artists.push(Artist {
                id: artist["id"].as_str().ok_or(Error::CreateError)?.to_owned(),
                name: artist["attributes"]["name"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
            })
        }

        let mut genres: Vec<String> = Vec::new();
        for genre in data["attributes"]["genreNames"]
            .as_array()
            .ok_or(Error::CreateError)?
        {
            genres.push(genre.as_str().ok_or(Error::CreateError)?.to_owned())
        }

        Ok(AppleMusic {
            id: data["id"].as_str().ok_or(Error::CreateError)?.to_owned(),
            artists,
            album: Album {
                id: data["relationships"]["albums"]["data"][0]["id"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
                name: data["relationships"]["albums"]["data"][0]["attributes"]["name"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
                total_tracks: data["relationships"]["albums"]["data"][0]["attributes"]
                    ["trackCount"]
                    .as_u64()
                    .ok_or(Error::CreateError)?
                    .try_into()
                    .or(Err(Error::CreateError))?,
                ean: match data["relationships"]["albums"]["data"][0]["attributes"]["ean"].as_str()
                {
                    Some(upc) => Some(upc.to_owned()),
                    None => None,
                },
                upc: match data["relationships"]["albums"]["data"][0]["attributes"]["upc"].as_str()
                {
                    Some(upc) => Some(upc.to_owned()),
                    None => None,
                },
            },
            url: data["attributes"]["url"]
                .as_str()
                .ok_or(Error::CreateError)?
                .to_owned(),
            image: match data["attributes"]["artwork"]["url"].as_str() {
                Some(url) => Some(url.to_owned()),
                None => None,
            },
            audio_preview: match data["attributes"]["previews"][0]["url"].as_str() {
                Some(url) => Some(url.to_owned()),
                None => None,
            },
            genres,
            composer: match data["attributes"]["composerName"].as_str() {
                Some(composer) => Some(composer.to_owned()),
                None => None,
            },
        })
    }

    async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track> {
        let mut artists: Vec<String> = Vec::new();
        for artist in data["relationships"]["artists"]["data"]
            .as_array()
            .ok_or(Error::CreateError)?
        {
            artists.push(
                artist["attributes"]["name"]
                    .as_str()
                    .ok_or(Error::CreateError)?
                    .to_owned(),
            )
        }

        let mut release_date: std::str::Split<&str> = data["attributes"]["releaseDate"]
            .as_str()
            .ok_or(Error::CreateError)?
            .split("-");

        Ok(Track {
            name: data["attributes"]["name"]
                .as_str()
                .ok_or(Error::CreateError)?
                .to_owned(),
            album: data["attributes"]["albumName"]
                .as_str()
                .ok_or(Error::CreateError)?
                .to_owned(),
            disk_number: data["attributes"]["discNumber"]
                .as_u64()
                .ok_or(Error::CreateError)?
                .try_into()
                .or(Err(Error::CreateError))?,
            track_number: data["attributes"]["trackNumber"]
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
            is_explicit: match data["attributes"]["contentRating"].as_str() {
                Some(content_rating) => {
                    if content_rating == "explicit" {
                        true
                    } else {
                        false
                    }
                }
                None => false,
            },
            duration_ms: data["attributes"]["durationInMillis"]
                .as_u64()
                .ok_or(Error::CreateError)?,
            services: Services {
                spotify: None,
                apple_music: Some(AppleMusic::create_service_from_raw(data).await?),
                youtube: None,
                bandcamp: None,
            },
            isrc: match data["attributes"]["isrc"].as_str() {
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
        let track_data: serde_json::Value = Self::get_raw(
            client,
            auth_token,
            &format!("catalog/us/songs/{}?include=artists,albums", track_id),
        )
        .await?;
        Ok(Self::create_track_from_raw(&track_data["data"][0]).await?)
    }

    async fn create_playlist_from_raw(data: &serde_json::Value) -> Result<crate::track::Playlist> {
        todo!()
    }

    async fn create_playlist_from_id(
        client: &Client,
        auth_token: &str,
        playlist_id: &str,
    ) -> Result<crate::track::Playlist> {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     // use crate::apple_music::AppleMusic;

//     #[test]
//     fn test_isrc_three_minute_hero_by_the_selector() {
//         dotenv::dotenv().ok();
//         let results = search_by_isrc("GBAYK8000001");
//         // assert!(t.n > 0)
//     }
// }
