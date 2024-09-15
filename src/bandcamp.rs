use crate::error::{Error, Result};
use crate::service::{Album, Artist};
use crate::track::Track;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Bandcamp {
    pub id: String,
    pub name: String,
    pub url: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub image: String,
    pub duration_ms: usize,
    pub streaming_url: Option<String>,
}

// pub struct RawTrack {
//     track: RawTrackSearchResult,
//     album: RawAlbum,
// }

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawTrackSearchResult {
    pub r#type: String,
    pub id: usize,
    pub name: String,
    pub band_id: usize,
    pub band_name: String,
    pub album_id: Option<usize>,
    pub album_name: Option<String>,
    pub art_id: Option<usize>,
    pub img_id: Option<usize>,
    pub img: Option<String>,
    pub item_url_root: String,
    pub item_url_path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawAlbum {
    pub id: usize,
    pub r#type: String,
    pub title: String,
    pub bandcamp_url: String,
    pub art_id: usize,
    pub band: RawAlbumBand,
    pub tralbum_artist: String,
    pub package_art: Value,
    pub tracks: Vec<RawAlbumTrack>,
    pub credits: Option<String>,
    pub album_id: Option<usize>,
    pub album_title: Option<String>,
    pub release_date: usize,
    pub is_preorder: bool,
    pub tags: Vec<Value>,
    pub label: Option<String>,
    pub label_id: Option<usize>,
    pub num_downloadable_tracks: usize,
    // pub featured_track_id: usize,
    // pub about: String,
    // pub is_purchasable: bool,
    // pub free_download: bool,
    // pub currency: String,
    // pub is_set_price: bool,
    // pub price: f64,
    // pub require_email: bool,
    // pub package_details_lite: Value,
    // pub has_digital_download: bool,
    // pub merch_sold_out: Option<bool>,
    // pub streaming_limit: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawAlbumBand {
    pub band_id: usize,
    pub name: String,
    pub image_id: Option<usize>,
    pub bio: Option<String>,
    pub location: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawAlbumTrack {
    pub track_id: usize,
    pub title: String,
    pub track_num: Option<usize>,
    pub streaming_url: Option<RawStreamingUrl>,
    pub duration: f64,
    pub encodings_id: usize,
    pub album_title: Option<String>,
    pub band_name: String,
    pub art_id: Option<usize>,
    pub album_id: Option<usize>,
    pub is_streamable: bool,
    pub has_lyrics: bool,
    pub band_id: usize,
    pub label: Option<String>,
    pub label_id: Option<usize>,
    pub track_license_id: Option<usize>,
    // pub is_set_price: bool,
    // pub price: f64,
    // pub has_digital_download: bool,
    // pub merch_ids: Option<Vec<usize>>,
    // pub merch_sold_out: Option<bool>,
    // pub currency: String,
    // pub require_email: bool,
    // pub is_purchasable: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawStreamingUrl {
    #[serde(rename(deserialize = "mp3-128"))]
    pub mp3_128: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct SearchPayload<'a> {
    search_text: &'a str,
    search_filter: &'a str,
    full_page: bool,
    fan_id: Option<usize>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct AlbumDetailsPayload<'a> {
    tralbum_id: usize,
    band_id: usize,
    tralbum_type: &'a str,
}

impl Bandcamp {
    pub const API_BASE_URL: &'static str = "https://bandcamp.com/api";
    pub const IMAGE_API_BASE_URL: &'static str = "https://f4.bcbits.com/img";

    pub async fn download(&self, client: &Client, path: &str, filename: &str) -> Result<()> {
        let request = match &self.streaming_url {
            Some(streaming_url) => client.get(streaming_url),
            None => {
                eprintln!("no streaming url");
                return Err(Error::DownloadError);
            }
        };

        let response = request.send().await?;

        let mut file = std::fs::File::create(format!("{}{}.mp3", path, filename))?;
        file.write_all(&response.bytes().await?)?;

        Ok(())
    }

    pub async fn post(client: &Client, path: &str, body: &str) -> Result<Value> {
        // let raw_payload: String = serde_json::to_string(&body)?;

        let request: RequestBuilder = client
            .post(format!("{}/{}", Self::API_BASE_URL, path))
            .body(body.to_owned());

        let mut response: Response = request.send().await?;
        response = response.error_for_status()?;

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    pub async fn get_raw_results_from_search(
        client: &Client,
        query: &str,
    ) -> Result<Vec<RawTrackSearchResult>> {
        let payload = SearchPayload {
            search_text: query,
            search_filter: "t",
            full_page: false,
            fan_id: None,
        };

        let mut results = Self::post(
            client,
            "bcsearch_public_api/1/autocomplete_elastic",
            &serde_json::to_string(&payload)?,
        )
        .await?;

        let tracks: Vec<RawTrackSearchResult> =
            serde_json::from_value(results["auto"]["results"].take())?;

        Ok(tracks)
    }

    pub async fn get_raw_album_from_id(
        client: &Client,
        track_id: usize,
        band_id: usize,
    ) -> Result<RawAlbum> {
        let payload = AlbumDetailsPayload {
            tralbum_id: track_id,
            band_id,
            tralbum_type: "t",
        };

        let mut results = Self::post(
            client,
            "mobile/25/tralbum_details",
            &serde_json::to_string(&payload)?,
        )
        .await?;

        // println!("{}", serde_json::to_string_pretty(&results)?);

        let album: RawAlbum = serde_json::from_value(results.take())?;

        Ok(album)
    }

    pub async fn get_raw_track_match_from_track(
        client: &Client,
        track: &Track,
    ) -> Result<RawAlbum> {
        let lackluster_search_result: Vec<RawTrackSearchResult> =
            Self::get_raw_results_from_search(
                client,
                &format!(
                    "{}, {}, {}",
                    track.name,
                    track.artists.get(0).ok_or(Error::MalformedTrackError)?,
                    track.album,
                ),
            )
            .await?;

        for raw_track_search_result in &lackluster_search_result {
            let album_name: &str = match &raw_track_search_result.album_name {
                Some(a) => a,
                None => {
                    // if their are multiple results, go to the next one which
                    // might have an album attached to it.
                    // if there are multiple tracks all with no albums, then
                    // nothing gets chosen...
                    if lackluster_search_result.len() > 1 {
                        continue;
                    }
                    ""
                }
            };

            if track.compare_similarity(
                &raw_track_search_result.name,
                &raw_track_search_result.band_name,
                album_name,
                0,
            ) >= 2
            {
                let raw_album: RawAlbum = Self::get_raw_album_from_id(
                    client,
                    raw_track_search_result.id,
                    raw_track_search_result.band_id,
                )
                .await?;

                return Ok(raw_album);
            }
        }

        Err(Error::MatchError)
    }

    pub async fn create_service_for_track(client: &Client, track: &mut Track) -> Result<()> {
        let raw_album: RawAlbum = Self::get_raw_track_match_from_track(client, track).await?;
        let service: Self = Self::create_service_from_raw(&raw_album).await?;
        track.services.bandcamp = Some(service);
        Ok(())
    }

    pub async fn create_service_from_raw(raw_album: &RawAlbum) -> Result<Self> {
        let track: &RawAlbumTrack = raw_album.tracks.get(0).ok_or(Error::CreateError)?;
        let track_url_split = raw_album.bandcamp_url.split("/");
        let artist_url: String = track_url_split.collect::<Vec<&str>>()[0..3].join("/");
        let album_url: String = match raw_album.album_id {
            Some(album_id) => format!("https://bandcamp.com/EmbeddedPlayer/album={}", album_id),
            None => "".to_owned(),
        };

        return Ok(Bandcamp {
            id: track.track_id.to_string(),
            name: track.title.to_owned(),
            url: raw_album.bandcamp_url.to_owned(),
            artists: vec![Artist {
                id: raw_album.band.band_id.to_string(),
                name: raw_album.band.name.to_owned(),
                url: artist_url,
            }],
            album: Album {
                id: raw_album.id.to_string(),
                name: raw_album.title.to_owned(),
                url: album_url,
                total_tracks: None,
                ean: None,
                upc: None,
            },
            image: format!("{}/a{}_0.jpg", Self::IMAGE_API_BASE_URL, raw_album.art_id),
            streaming_url: match &track.streaming_url {
                Some(url) => Some(url.mp3_128.to_owned()),
                None => None,
            },
            duration_ms: (track.duration * 1000_f64) as usize,
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::{service::Services, track::Track};

    #[tokio::test]
    async fn get_match() {
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

        example_track.add_bandcamp(&client).await.unwrap();
    }
}
