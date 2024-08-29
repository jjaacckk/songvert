use crate::error::{Error, Result};
use crate::service::{Album, Artist, Services};
use crate::track::{Playlist, Track};
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
    pub const API_BASE_URL: &'static str = "https://api.music.apple.com/v1";
    pub const SITE_BASE_URL: &'static str = "https://music.apple.com";

    pub const PUBLIC_BEARER_TOKEN: &'static str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldlYlBsYXlLaWQifQ.eyJpc3MiOiJBTVBXZWJQbGF5IiwiaWF0IjoxNzIxNzczNjI0LCJleHAiOjE3MjkwMzEyMjQsInJvb3RfaHR0cHNfb3JpZ2luIjpbImFwcGxlLmNvbSJdfQ.cMMhHLLazlxgiIbwBSSP1YuHCgqAVxiF7UQrwBc5xZepWt-vjqth_o4BidXFrmsEJvwzZKJ-GAMbqJpIeGcl7w";

    pub async fn get(client: &Client, auth: &str, path: &str) -> Result<serde_json::Value> {
        let request: RequestBuilder = client
            .get(format!("{}/{}", Self::API_BASE_URL, path))
            .header("Authorization", format!("Bearer {}", auth))
            .header("Origin", Self::SITE_BASE_URL);
        let response: Response = request.send().await?;
        if response.status() != 200 {
            eprintln!("{}", response.text().await?);
            return Err(Error::FindError);
        }

        let data: serde_json::Value = serde_json::from_str(&response.text().await?)?;

        Ok(data)
    }

    pub async fn get_raw_track_match_from_track(
        client: &Client,
        auth: &str,
        track: &Track,
    ) -> Result<serde_json::Value> {
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
                Ok(raw_data) => {
                    // eprintln!("{}", raw_data);

                    let raw_tracks: &Vec<serde_json::Value> =
                        raw_data["data"].as_array().ok_or(Error::MatchError)?;

                    // eprintln!("array length: {}", raw_tracks.len());

                    if raw_tracks.len() == 1 {
                        return Ok(raw_data["data"][0].to_owned());
                    } else if raw_tracks.len() > 1 {
                        // check album name
                        for raw_track in raw_tracks {
                            // println!(
                            //     "{}\n{}",
                            //     raw_track["attributes"]["albumName"]
                            //         .as_str()
                            //         .ok_or(Error::MatchError)?,
                            //     track.album
                            // );

                            if raw_track["attributes"]["albumName"]
                                .as_str()
                                .ok_or(Error::MatchError)?
                                == track.album
                            {
                                return Ok(raw_track.to_owned());
                            }
                        }

                        // fallback: can't find track with same album name
                        return Ok(raw_data["data"][0].to_owned());
                    }
                }
                Err(..) => (),
            },
            None => (),
        }
        // no isrc or isrc search failed
        eprintln!("isrc search failed....fallback....");

        let lackluster_search_result: serde_json::Value = Self::get(
            &client,
            &auth,
            &format!(
                "catalog/us/search?types=songs&term=track:{}%20artist:{}%20album:{}%20year:{}",
                &track.name,
                &track.artists.join("+"),
                &track.album,
                &track.release_year
            )
            .replace(" ", "+"),
        )
        .await?;

        match Self::get(
            &client,
            &auth,
            &format!(
                "catalog/us/songs/{}?include=artists,albums",
                lackluster_search_result["results"]["songs"]["data"][0]["id"]
                    .as_str()
                    .ok_or(Error::MatchError)?
            ),
        )
        .await
        {
            Ok(raw_track) => Ok(raw_track["data"][0].to_owned()),
            Err(e) => Err(e),
        }
    }

    pub async fn create_service_for_track(
        client: &Client,
        auth: &str,
        track: &mut Track,
    ) -> Result<()> {
        let data: serde_json::Value =
            Self::get_raw_track_match_from_track(client, auth, track).await?;
        let service: Self = Self::create_service_from_raw(&data).await?;
        track.services.apple_music = Some(service);
        Ok(())
    }

    pub async fn create_track_from_id(
        client: &Client,
        auth: &str,
        track_id: &str,
    ) -> Result<Track> {
        let track_data: serde_json::Value = Self::get(
            client,
            auth,
            &format!("catalog/us/songs/{}?include=artists,albums", track_id),
        )
        .await?;
        Ok(Self::create_track_from_raw(&track_data["data"][0]).await?)
    }

    pub async fn create_playlist_from_id(
        client: &Client,
        auth: &str,
        playlist_id: &str,
    ) -> Result<Playlist> {
        todo!()
    }

    pub async fn create_service_from_raw(data: &serde_json::Value) -> Result<Self>
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

    pub async fn create_track_from_raw(data: &serde_json::Value) -> Result<Track> {
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
            // duration_ms: data["attributes"]["durationInMillis"],
            duration_ms: 0,
            services: Services {
                spotify: None,
                apple_music: Some(Self::create_service_from_raw(data).await?),
                youtube: None,
                bandcamp: None,
            },
            isrc: match data["attributes"]["isrc"].as_str() {
                Some(isrc) => Some(isrc.to_owned()),
                None => None,
            },
        })
    }

    pub async fn create_playlist_from_raw(data: &Vec<serde_json::Value>) -> Result<Playlist> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        apple_music::AppleMusic,
        service::Services,
        track::{Playlist, Track},
    };
    #[tokio::test]
    async fn good_isrc_search() {
        let good_isrc: &str = "GBAYK8000001";
        let client: reqwest::Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()
            .unwrap();

        let search_result: serde_json::Value = AppleMusic::get(
            &client,
            AppleMusic::PUBLIC_BEARER_TOKEN,
            &format!(
                "catalog/us/songs?filter[isrc]={}&include=albums,artists",
                good_isrc
            ),
        )
        .await
        .unwrap()["data"][0]
            .to_owned();

        assert_eq!(
        search_result,
        serde_json::from_str::<serde_json::Value>(
            r#"{"attributes":{"albumName":"Selected Selections","artistName":"The Selecter","artwork":{"bgColor":"241b2c","height":1400,"textColor1":"e4dce4","textColor2":"f69b55","textColor3":"bdb5bf","textColor4":"cc814d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/27/e0/cd/27e0cd82-68eb-8405-1956-09e9573f937b/5054526576636.png/{w}x{h}bb.jpg","width":1400},"discNumber":1,"durationInMillis":183273,"genreNames":["Pop","Music","Alternative","New Wave","Reggae","Ska"],"hasLyrics":true,"isAppleDigitalMaster":false,"isrc":"GBAYK8000001","name":"Three Minute Hero","playParams":{"id":"1629179587","kind":"song"},"previews":[{"url":"https://audio-ssl.itunes.apple.com/itunes-assets/AudioPreview122/v4/a4/28/43/a42843a4-19c2-5c2a-ac37-41bb77d44a20/mzaf_7773402356583530628.plus.aac.p.m4a"}],"releaseDate":"1980-02-23","trackNumber":10,"url":"https://music.apple.com/us/album/three-minute-hero/1629179487?i=1629179587"},"href":"/v1/catalog/us/songs/1629179587","id":"1629179587","relationships":{"albums":{"data":[{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"241b2c","height":1400,"textColor1":"e4dce4","textColor2":"f69b55","textColor3":"bdb5bf","textColor4":"cc814d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/27/e0/cd/27e0cd82-68eb-8405-1956-09e9573f937b/5054526576636.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 1989 Chrysalis Records Limited","genreNames":["Pop","Music","Alternative","New Wave","Reggae","Ska"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Selected Selections","playParams":{"id":"1629179487","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1989-08-30","trackCount":14,"upc":"5054526576636","url":"https://music.apple.com/us/album/selected-selections/1629179487"},"href":"/v1/catalog/us/albums/1629179487","id":"1629179487","type":"albums"}],"href":"/v1/catalog/us/songs/1629179587/albums"},"artists":{"data":[{"attributes":{"artwork":{"bgColor":"f2ebf2","height":2400,"textColor1":"0f0d10","textColor2":"2c261e","textColor3":"3c3a3d","textColor4":"534d48","url":"https://is1-ssl.mzstatic.com/image/thumb/Features221/v4/80/53/e5/8053e59a-5d55-e6db-5732-2ac7864bd273/mza_13640949984455360988.png/{w}x{h}bb.jpg","width":2400},"genreNames":["Reggae"],"name":"The Selecter","url":"https://music.apple.com/us/artist/the-selecter/541984"},"href":"/v1/catalog/us/artists/541984","id":"541984","relationships":{"albums":{"data":[{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"110f12","height":1500,"textColor1":"f5f3fe","textColor2":"e1e9f7","textColor3":"c7c5ce","textColor4":"b7bec9","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/a4/47/90/a44790f8-16de-7bc6-51b2-d9d2dc26ff8d/5054526648791.png/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 1996 Chrysalis Records Limited","genreNames":["Pop","Music","Reggae","Ska","Alternative","New Wave"],"isCompilation":true,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Greatest Hits","playParams":{"id":"1629188190","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1996-05-13","trackCount":16,"upc":"5054526648791","url":"https://music.apple.com/us/album/greatest-hits/1629188190"},"href":"/v1/catalog/us/albums/1629188190","id":"1629188190","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"ffffff","height":3000,"textColor1":"090909","textColor2":"202020","textColor3":"3a3a3a","textColor4":"4d4d4d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music122/v4/02/61/37/026137fb-ae6f-26d6-0e76-b4296ca851d9/5060516097722.png/{w}x{h}bb.jpg","width":3000},"copyright":"℗ 2021 Chrysalis Records Limited","genreNames":["Pop","Music","Alternative","New Wave","Reggae","Ska"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Too Much Pressure [Deluxe Edition]","playParams":{"id":"1629522932","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1980-02","trackCount":46,"upc":"5060516097722","url":"https://music.apple.com/us/album/too-much-pressure-deluxe-edition/1629522932"},"href":"/v1/catalog/us/albums/1629522932","id":"1629522932","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"e0e0d4","height":2400,"textColor1":"030605","textColor2":"212424","textColor3":"2f312e","textColor4":"474947","url":"https://is1-ssl.mzstatic.com/image/thumb/Music122/v4/06/ca/9e/06ca9e83-b390-7cd9-382a-6aee2b215827/5054526646513.png/{w}x{h}bb.jpg","width":2400},"copyright":"℗ 1980 Chrysalis Records Limited","genreNames":["Reggae","Music","Ska","Alternative","New Wave"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Too Much Pressure","playParams":{"id":"1629178476","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1980-02-23","trackCount":13,"upc":"5054526646513","url":"https://music.apple.com/us/album/too-much-pressure/1629178476"},"href":"/v1/catalog/us/albums/1629178476","id":"1629178476","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"fffffe","height":1500,"textColor1":"090d0c","textColor2":"212826","textColor3":"3a3e3c","textColor4":"4e5351","url":"https://is1-ssl.mzstatic.com/image/thumb/Music125/v4/a3/16/a8/a316a88d-2364-c5d9-fbfe-6d3e5b1e733f/mzi.faqnqrku.tif/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 1997 Guava Jelly Inc.","genreNames":["New Wave","Music","Alternative","Reggae","Ska"],"isCompilation":true,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"The Very Best of The Selecter","playParams":{"id":"417271887","kind":"album"},"recordLabel":"Triple X Records","releaseDate":"1996","trackCount":20,"upc":"885686538555","url":"https://music.apple.com/us/album/the-very-best-of-the-selecter/417271887"},"href":"/v1/catalog/us/albums/417271887","id":"417271887","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"ffffff","height":1650,"textColor1":"080d02","textColor2":"3c240e","textColor3":"393d35","textColor4":"634f3e","url":"https://is1-ssl.mzstatic.com/image/thumb/Music122/v4/99/67/e3/9967e384-4451-8a18-62bd-70c81f55a564/5054526644960.png/{w}x{h}bb.jpg","width":1650},"copyright":"℗ 1981 Chrysalis Records Limited","genreNames":["Pop","Music","Reggae","Ska","Alternative","New Wave"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Celebrate the Bullet","playParams":{"id":"1629184086","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1981-01-01","trackCount":11,"upc":"5054526644960","url":"https://music.apple.com/us/album/celebrate-the-bullet/1629184086"},"href":"/v1/catalog/us/albums/1629184086","id":"1629184086","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"fbb956","height":1400,"textColor1":"0f0b05","textColor2":"1a1509","textColor3":"3e2e15","textColor4":"473618","url":"https://is1-ssl.mzstatic.com/image/thumb/Music128/v4/ee/88/e5/ee88e529-cd34-9fb3-2a4d-545d47afdbc1/191773530617.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2017 DMF Records","editorialNotes":{"short":"The second-wave ska legends “pick it up” where they left off."},"genreNames":["Reggae","Music","Ska","Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Daylight","playParams":{"id":"1446576449","kind":"album"},"recordLabel":"DMF Records","releaseDate":"2017-10-06","trackCount":10,"upc":"191773530617","url":"https://music.apple.com/us/album/daylight/1446576449"},"href":"/v1/catalog/us/albums/1446576449","id":"1446576449","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"241b2c","height":1400,"textColor1":"e4dce4","textColor2":"f69b55","textColor3":"bdb5bf","textColor4":"cc814d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/27/e0/cd/27e0cd82-68eb-8405-1956-09e9573f937b/5054526576636.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 1989 Chrysalis Records Limited","genreNames":["Pop","Music","Alternative","New Wave","Reggae","Ska"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Selected Selections","playParams":{"id":"1629179487","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1989-08-30","trackCount":14,"upc":"5054526576636","url":"https://music.apple.com/us/album/selected-selections/1629179487"},"href":"/v1/catalog/us/albums/1629179487","id":"1629179487","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"d1d0cc","height":1500,"textColor1":"010100","textColor2":"171717","textColor3":"2b2b29","textColor4":"3c3c3b","url":"https://is1-ssl.mzstatic.com/image/thumb/Music7/v4/94/33/c8/9433c840-d205-7397-30d2-76d1bd9d777a/634457445444.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2015 DMF Records","genreNames":["Reggae","Music","Rock","Ska"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Subculture","playParams":{"id":"1011473513","kind":"album"},"recordLabel":"DMF Records","releaseDate":"2015-10-02","trackCount":12,"upc":"634457445444","url":"https://music.apple.com/us/album/subculture/1011473513"},"href":"/v1/catalog/us/albums/1011473513","id":"1011473513","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"e7dbcb","height":3700,"textColor1":"2d1f18","textColor2":"512819","textColor3":"52443c","textColor4":"6f4c3d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music122/v4/95/d1/71/95d17118-27b2-fa56-669d-d1565b519561/5060516099696.png/{w}x{h}bb.jpg","width":3700},"copyright":"℗ 2022 Chrysalis Records Limited","genreNames":["Ska","Music","Reggae","Alternative","New Wave"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Celebrate the Bullet (Deluxe Edition) [2022 Remaster]","playParams":{"id":"1642129805","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1981-03","trackCount":37,"upc":"5060516099696","url":"https://music.apple.com/us/album/celebrate-the-bullet-deluxe-edition-2022-remaster/1642129805"},"href":"/v1/catalog/us/albums/1642129805","id":"1642129805","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"ffffff","height":1400,"textColor1":"171717","textColor2":"2e2e2e","textColor3":"464646","textColor4":"585858","url":"https://is1-ssl.mzstatic.com/image/thumb/Music118/v4/e5/eb/54/e5eb541f-efb2-1bcc-c969-7d2c75f5e7fe/5050749410467.jpg/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2004 Sanctuary Records Group Ltd, a BMG company","genreNames":["Pop","Music","Reggae","Ska"],"isCompilation":true,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Street Feeling","playParams":{"id":"1439693346","kind":"album"},"recordLabel":"Trojan Records","releaseDate":"2005-02-08","trackCount":34,"upc":"5050749410467","url":"https://music.apple.com/us/album/street-feeling/1439693346"},"href":"/v1/catalog/us/albums/1439693346","id":"1439693346","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"010101","height":600,"textColor1":"ffffff","textColor2":"fe0000","textColor3":"cccccc","textColor4":"cb0000","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/85/29/e9/mzi.lyrpiswe.jpg/{w}x{h}bb.jpg","width":600},"contentRating":"explicit","copyright":"℗ 2011 vocaphone","genreNames":["Reggae","Music","Worldwide"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Made In Britain","playParams":{"id":"463871744","kind":"album"},"recordLabel":"vocaphone","releaseDate":"2011-09-04","trackCount":10,"upc":"885767851665","url":"https://music.apple.com/us/album/made-in-britain/463871744"},"href":"/v1/catalog/us/albums/463871744","id":"463871744","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"000000","height":3000,"textColor1":"ffffff","textColor2":"f51d24","textColor3":"cbcbcb","textColor4":"c4171d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music126/v4/d5/3f/02/d53f02ac-50dc-9ed4-73eb-56800516ef0d/5053760102564_cover.jpg/{w}x{h}bb.jpg","width":3000},"copyright":"℗ 2023 DMF Records","genreNames":["Ska","Music","Reggae"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":true,"isSingle":false,"name":"Human Algebra","playParams":{"id":"1673519075","kind":"album"},"recordLabel":"DMF Records","releaseDate":"2023-05-19","trackCount":12,"upc":"5053760102564","url":"https://music.apple.com/us/album/human-algebra/1673519075"},"href":"/v1/catalog/us/albums/1673519075","id":"1673519075","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"ffffff","height":1400,"textColor1":"020102","textColor2":"100e0e","textColor3":"353434","textColor4":"3f3e3e","url":"https://is1-ssl.mzstatic.com/image/thumb/Music118/v4/89/51/59/895159cf-3829-9a1a-31f2-498c4260e592/192562324110.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2018 DMF Records","genreNames":["Ska","Music","Reggae"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Live at the Roundhouse","playParams":{"id":"1446570449","kind":"album"},"recordLabel":"DMF Records","releaseDate":"2018-06-08","trackCount":14,"upc":"192562324110","url":"https://music.apple.com/us/album/live-at-the-roundhouse/1446570449"},"href":"/v1/catalog/us/albums/1446570449","id":"1446570449","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"0d0802","height":3000,"textColor1":"ffffff","textColor2":"e57c3f","textColor3":"cecdcc","textColor4":"ba6533","url":"https://is1-ssl.mzstatic.com/image/thumb/Music118/v4/ff/b4/c4/ffb4c482-65f8-34fa-6955-815ddf15bd05/The_Selecter.jpg/{w}x{h}bb.jpg","width":3000},"copyright":"℗ 2018 Secret Records","genreNames":["Ska","Music","Reggae","Rock"],"isCompilation":true,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Best of Live At Dingwalls London","playParams":{"id":"1353503087","kind":"album"},"recordLabel":"Secret Records","releaseDate":"2018-03-23","trackCount":9,"upc":"5036436113521","url":"https://music.apple.com/us/album/best-of-live-at-dingwalls-london/1353503087"},"href":"/v1/catalog/us/albums/1353503087","id":"1353503087","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"ffffff","height":3000,"textColor1":"000000","textColor2":"242424","textColor3":"333333","textColor4":"505050","url":"https://is1-ssl.mzstatic.com/image/thumb/Music116/v4/51/1d/b0/511db06e-efb8-2726-63a0-c5d1cc36c3ee/5053760103912_cover.jpg/{w}x{h}bb.jpg","width":3000},"copyright":"℗ 2023 DMF Records","genreNames":["Reggae","Music"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":true,"name":"War War War - Single","playParams":{"id":"1678301208","kind":"album"},"recordLabel":"DMF Records","releaseDate":"2023-04-14","trackCount":1,"upc":"5053760103912","url":"https://music.apple.com/us/album/war-war-war-single/1678301208"},"href":"/v1/catalog/us/albums/1678301208","id":"1678301208","type":"albums"},{"attributes":{"artistName":"The Specials & The Selecter","artwork":{"bgColor":"e2e2d8","height":3000,"textColor1":"040404","textColor2":"252525","textColor3":"31312e","textColor4":"4b4b49","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/92/b1/bf/92b1bfaf-5560-9a74-c070-ebe0f9365855/5060516099245.png/{w}x{h}bb.jpg","width":3000},"copyright":"℗ 2022 Chrysalis Records Limited","genreNames":["Ska","Music","Reggae"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Gangsters / The Selecter (2022 Remaster) - Single","playParams":{"id":"1629090428","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"1979-01-01","trackCount":2,"upc":"5060516099245","url":"https://music.apple.com/us/album/gangsters-the-selecter-2022-remaster-single/1629090428"},"href":"/v1/catalog/us/albums/1629090428","id":"1629090428","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"edecea","height":1500,"textColor1":"00090d","textColor2":"291d27","textColor3":"2f3639","textColor4":"50464e","url":"https://is1-ssl.mzstatic.com/image/thumb/Music2/v4/d3/75/ae/d375ae37-c049-6310-64fc-40842738d96f/888831237143.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2013 Badfish Records","genreNames":["Ska","Music","Reggae","Rock"],"isCompilation":true,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Indie Singles Collection 1991-96","playParams":{"id":"886364314","kind":"album"},"recordLabel":"Badfish Records","releaseDate":"2014-06-09","trackCount":38,"upc":"888831237143","url":"https://music.apple.com/us/album/indie-singles-collection-1991-96/886364314"},"href":"/v1/catalog/us/albums/886364314","id":"886364314","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"486063","height":1406,"textColor1":"f9fffd","textColor2":"edf8fe","textColor3":"d5dfde","textColor4":"ccdadf","url":"https://is1-ssl.mzstatic.com/image/thumb/Music124/v4/2d/e2/f0/2de2f03e-7493-d149-19b3-5cb994184dc8/5060243323439.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2015 DMF Records","genreNames":["Ska","Music","Reggae"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":true,"name":"Walk the Walk - Single","playParams":{"id":"1448273674","kind":"album"},"recordLabel":"DMF Records","releaseDate":"2015-08-14","trackCount":1,"upc":"5060243323439","url":"https://music.apple.com/us/album/walk-the-walk-single/1448273674"},"href":"/v1/catalog/us/albums/1448273674","id":"1448273674","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"251b24","height":1500,"textColor1":"f0f1f3","textColor2":"cbc3d3","textColor3":"c7c6ca","textColor4":"aaa1b0","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/64/98/ad/6498adde-e910-d9e0-3506-448eed64a184/888831330165.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2004 Moon Ska Europe","genreNames":["Ska","Music","Reggae","Alternative","New Wave"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Cruel Britannia","playParams":{"id":"891000224","kind":"album"},"recordLabel":"Moon Ska Europe","releaseDate":"1999-02-23","trackCount":11,"upc":"888831330165","url":"https://music.apple.com/us/album/cruel-britannia/891000224"},"href":"/v1/catalog/us/albums/891000224","id":"891000224","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"000004","height":600,"textColor1":"e2e3e8","textColor2":"e5e1ed","textColor3":"b4b5bb","textColor4":"b7b4be","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/98/60/f6/mzi.janwvvtd.jpg/{w}x{h}bb.jpg","width":600},"copyright":"℗ 2009 Secret Records Limited","genreNames":["Reggae","Music"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"In London","playParams":{"id":"323745206","kind":"album"},"recordLabel":"Secret Records Limited","releaseDate":"2009-07-31","trackCount":16,"upc":"5036436030026","url":"https://music.apple.com/us/album/in-london/323745206"},"href":"/v1/catalog/us/albums/323745206","id":"323745206","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"a4b1c4","height":1400,"textColor1":"030104","textColor2":"121212","textColor3":"23242a","textColor4":"2f3135","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/15/37/32/15373253-35f9-fc7a-d884-e3d29ed68ad5/5054526646537.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2009 Chrysalis Records Limited","genreNames":["Pop","Music"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"John Peel Session (9 October 1979) - EP","playParams":{"id":"1629182916","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"2009-12-04","trackCount":4,"upc":"5054526646537","url":"https://music.apple.com/us/album/john-peel-session-9-october-1979-ep/1629182916"},"href":"/v1/catalog/us/albums/1629182916","id":"1629182916","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"010101","height":1400,"textColor1":"ffffff","textColor2":"d0d0d0","textColor3":"cccccc","textColor4":"a6a6a6","url":"https://is1-ssl.mzstatic.com/image/thumb/Music2/v4/81/33/db/8133db8e-2986-f711-0fa2-4f2b4e7670d0/887516199691.jpg/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2013 Vocaphone","genreNames":["Reggae","Music","Worldwide"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"String Theory","playParams":{"id":"598191592","kind":"album"},"recordLabel":"Vocaphone","releaseDate":"2013-02-24","trackCount":10,"upc":"887516199691","url":"https://music.apple.com/us/album/string-theory/598191592"},"href":"/v1/catalog/us/albums/598191592","id":"598191592","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"091118","height":1500,"textColor1":"86ca50","textColor2":"e4754b","textColor3":"6da544","textColor4":"b96141","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/12/e5/67/mzi.dfrioeqh.tif/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2008 One Media Publishing","genreNames":["Reggae","Music","Ska","Alternative","New Wave"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Live At Roskilde","playParams":{"id":"280504745","kind":"album"},"recordLabel":"Synergie OMP","releaseDate":"1996-12-17","trackCount":14,"upc":"884385060626","url":"https://music.apple.com/us/album/live-at-roskilde/280504745"},"href":"/v1/catalog/us/albums/280504745","id":"280504745","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"161510","height":1500,"textColor1":"faeceb","textColor2":"e8e9eb","textColor3":"ccc1bf","textColor4":"bebebf","url":"https://is1-ssl.mzstatic.com/image/thumb/Music126/v4/43/c6/b0/43c6b059-dda1-f993-87db-cd57ee2d6ca8/5013929002609.jpg/{w}x{h}bb.jpg","width":1500},"contentRating":"explicit","copyright":"℗ 1994 Link","genreNames":["Rock","Music","Reggae","Ska","Alternative","Punk"],"isCompilation":true,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Rare, Vol. 1","playParams":{"id":"1605858067","kind":"album"},"recordLabel":"Right Honourable Records","releaseDate":"1994-01-01","trackCount":12,"upc":"5013929002609","url":"https://music.apple.com/us/album/rare-vol-1/1605858067"},"href":"/v1/catalog/us/albums/1605858067","id":"1605858067","type":"albums"},{"attributes":{"artistName":"The Selecter","artwork":{"bgColor":"dcd8d9","height":1400,"textColor1":"050304","textColor2":"362e38","textColor3":"302d2f","textColor4":"575058","url":"https://is1-ssl.mzstatic.com/image/thumb/Music112/v4/fb/8c/a3/fb8ca33b-3daa-fd16-c73b-7dfbcec0de76/5054526646711.png/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2009 Chrysalis Records Limited","genreNames":["Pop","Music"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Bbc in Concert (15 December 1979)","playParams":{"id":"1629116013","kind":"album"},"recordLabel":"Chrysalis Records","releaseDate":"2009-12-04","trackCount":8,"upc":"5054526646711","url":"https://music.apple.com/us/album/bbc-in-concert-15-december-1979/1629116013"},"href":"/v1/catalog/us/albums/1629116013","id":"1629116013","type":"albums"}],"href":"/v1/catalog/us/artists/541984/albums","next":"/v1/catalog/us/artists/541984/albums?offset=25"}},"type":"artists"}],"href":"/v1/catalog/us/songs/1629179587/artists"}},"type":"songs"}"#
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

        if AppleMusic::get(
            &client,
            AppleMusic::PUBLIC_BEARER_TOKEN,
            &format!(
                "catalog/us/songs?filter[isrc]={}&include=albums,artists",
                bad_isrc
            ),
        )
        .await
        .unwrap()["data"][0]
            .is_null()
            == false
        {
            panic!()
        }
    }

    #[tokio::test]
    async fn get_match_with_isrc() {
        let example_services: Services = Services {
            spotify: None,
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

        let client: reqwest::Client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                .build()
                .unwrap();

        let search_result: serde_json::Value = AppleMusic::get_raw_track_match_from_track(
            &client,
            &AppleMusic::PUBLIC_BEARER_TOKEN,
            &example_track,
        )
        .await
        .unwrap();

        assert_eq!(search_result, serde_json::from_str::<serde_json::Value>(r#"{"attributes":{"albumName":"Genius Fatigue","artistName":"TunaBunny","artwork":{"bgColor":"f1f0ee","height":1400,"textColor1":"000a13","textColor2":"000c11","textColor3":"30383e","textColor4":"30393d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/ec/33/ed/ec33ed1a-0cf5-1478-41eb-51ce063f92d6/Cover.jpg/{w}x{h}bb.jpg","width":1400},"discNumber":1,"durationInMillis":138027,"genreNames":["Alternative","Music","Rock","Pop","Pop/Rock","Indie Rock","College Rock"],"hasLyrics":false,"isAppleDigitalMaster":false,"isrc":"USZUD1215001","name":"Duchess for Nothing","playParams":{"id":"575329663","kind":"song"},"previews":[{"url":"https://audio-ssl.itunes.apple.com/itunes-assets/Music7/v4/20/62/e5/2062e57f-b30c-f22f-d926-0e224de8cee0/mzaf_8064208449823682309.plus.aac.p.m4a"}],"releaseDate":"2013-01-15","trackNumber":1,"url":"https://music.apple.com/us/album/duchess-for-nothing/575329457?i=575329663"},"href":"/v1/catalog/us/songs/575329663","id":"575329663","relationships":{"albums":{"data":[{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"f1f0ee","height":1400,"textColor1":"000a13","textColor2":"000c11","textColor3":"30383e","textColor4":"30393d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/ec/33/ed/ec33ed1a-0cf5-1478-41eb-51ce063f92d6/Cover.jpg/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2012 HHBTM","genreNames":["Alternative","Music","Rock","College Rock","Pop","Pop/Rock","Indie Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Genius Fatigue","playParams":{"id":"575329457","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2013-01-15","trackCount":10,"upc":"795103607620","url":"https://music.apple.com/us/album/genius-fatigue/575329457"},"href":"/v1/catalog/us/albums/575329457","id":"575329457","type":"albums"}],"href":"/v1/catalog/us/songs/575329663/albums"},"artists":{"data":[{"attributes":{"artwork":{"bgColor":"bc0068","height":620,"textColor1":"ffffff","textColor2":"f5e7ef","textColor3":"f1cbe0","textColor4":"eab9d4","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/93/a3/fe/mzi.nmmldnsd.jpg/{w}x{h}ac.jpg","width":620},"genreNames":["Alternative"],"name":"TunaBunny","url":"https://music.apple.com/us/artist/tunabunny/333943109"},"href":"/v1/catalog/us/artists/333943109","id":"333943109","relationships":{"albums":{"data":[{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"ee993f","height":1500,"textColor1":"0a0907","textColor2":"191712","textColor3":"372612","textColor4":"43311b","url":"https://is1-ssl.mzstatic.com/image/thumb/Music111/v4/3e/ef/e8/3eefe81a-c8fb-70db-a7ea-a54bebdcfa46/191018833985.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2017 HHBTM Records","genreNames":["Rock","Music","Alternative","College Rock","Electronic","Pop","Pop/Rock","Indie Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Pcp Presents Alice in Wonderland Jr.","playParams":{"id":"1205767123","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2017-06-09","trackCount":28,"upc":"191018833985","url":"https://music.apple.com/us/album/pcp-presents-alice-in-wonderland-jr/1205767123"},"href":"/v1/catalog/us/albums/1205767123","id":"1205767123","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"bc0068","height":620,"textColor1":"ffffff","textColor2":"f5e7ef","textColor3":"f1cbe0","textColor4":"eab9d4","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/93/a3/fe/mzi.nmmldnsd.jpg/{w}x{h}bb.jpg","width":620},"contentRating":"clean","copyright":"℗ 2011 HHBTM","genreNames":["Indie Rock","Music","Alternative","College Rock","Rock","Pop","Pop/Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Minima Moralia","playParams":{"id":"464244644","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2011-09-27","trackCount":12,"upc":"847108031723","url":"https://music.apple.com/us/album/minima-moralia/464244644"},"href":"/v1/catalog/us/albums/464244644","id":"464244644","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"f1f0ee","height":1400,"textColor1":"000a13","textColor2":"000c11","textColor3":"30383e","textColor4":"30393d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/ec/33/ed/ec33ed1a-0cf5-1478-41eb-51ce063f92d6/Cover.jpg/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2012 HHBTM","genreNames":["Alternative","Music","Rock","College Rock","Pop","Pop/Rock","Indie Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Genius Fatigue","playParams":{"id":"575329457","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2013-01-15","trackCount":10,"upc":"795103607620","url":"https://music.apple.com/us/album/genius-fatigue/575329457"},"href":"/v1/catalog/us/albums/575329457","id":"575329457","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"ffffff","height":620,"textColor1":"000000","textColor2":"161616","textColor3":"333333","textColor4":"454545","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/70/10/bf/mzi.tkuuzput.jpg/{w}x{h}bb.jpg","width":620},"copyright":"℗ 2010 Happy Happy Birthday to Me Records","genreNames":["Indie Rock","Music","Alternative","College Rock","Pop","Pop/Rock","Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Tunabunny","playParams":{"id":"377527201","kind":"album"},"recordLabel":"Happy Happy Birthday to Me Records","releaseDate":"2010-07-20","trackCount":15,"upc":"844185050173","url":"https://music.apple.com/us/album/tunabunny/377527201"},"href":"/v1/catalog/us/albums/377527201","id":"377527201","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"f48a21","height":1500,"textColor1":"090404","textColor2":"400c0c","textColor3":"381f0a","textColor4":"642510","url":"https://is1-ssl.mzstatic.com/image/thumb/Music4/v4/03/a7/e1/03a7e1fd-e31e-1a1a-1ca5-a6845953886c/888608550956.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2014 HHBTM Records","genreNames":["Alternative","Music","Electronic","Pop","Pop/Rock","Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Kingdom Technology","playParams":{"id":"818880961","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2014-03-11","trackCount":14,"upc":"888608550956","url":"https://music.apple.com/us/album/kingdom-technology/818880961"},"href":"/v1/catalog/us/albums/818880961","id":"818880961","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"c7c2bc","height":620,"textColor1":"000f30","textColor2":"111b33","textColor3":"28334c","textColor4":"353c4f","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/62/72/0f/mzi.etiucsro.jpg/{w}x{h}bb.jpg","width":620},"copyright":"℗ 2009 Happy Happy Birthday to Me Records","genreNames":["Alternative","Music"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":true,"name":"Outerspace Is the Center of the Earth - Single","playParams":{"id":"333943019","kind":"album"},"recordLabel":"Happy Happy Birthday to Me Records","releaseDate":"2009-09-15","trackCount":1,"upc":"844185068789","url":"https://music.apple.com/us/album/outerspace-is-the-center-of-the-earth-single/333943019"},"href":"/v1/catalog/us/albums/333943019","id":"333943019","type":"albums"}],"href":"/v1/catalog/us/artists/333943109/albums"}},"type":"artists"}],"href":"/v1/catalog/us/songs/575329663/artists"}},"type":"songs"}"#).unwrap());
    }

    #[tokio::test]
    async fn get_match_no_isrc() {
        let example_services: Services = Services {
            spotify: None,
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
            isrc: None,
        };

        let client: reqwest::Client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
                .build()
                .unwrap();

        let search_result: serde_json::Value = AppleMusic::get_raw_track_match_from_track(
            &client,
            &AppleMusic::PUBLIC_BEARER_TOKEN,
            &example_track,
        )
        .await
        .unwrap();

        assert_eq!(search_result, serde_json::from_str::<serde_json::Value>(r#"{"attributes":{"albumName":"Genius Fatigue","artistName":"TunaBunny","artwork":{"bgColor":"f1f0ee","height":1400,"textColor1":"000a13","textColor2":"000c11","textColor3":"30383e","textColor4":"30393d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/ec/33/ed/ec33ed1a-0cf5-1478-41eb-51ce063f92d6/Cover.jpg/{w}x{h}bb.jpg","width":1400},"discNumber":1,"durationInMillis":138027,"genreNames":["Alternative","Music","Rock","Pop","Pop/Rock","Indie Rock","College Rock"],"hasLyrics":false,"isAppleDigitalMaster":false,"isrc":"USZUD1215001","name":"Duchess for Nothing","playParams":{"id":"575329663","kind":"song"},"previews":[{"url":"https://audio-ssl.itunes.apple.com/itunes-assets/Music7/v4/20/62/e5/2062e57f-b30c-f22f-d926-0e224de8cee0/mzaf_8064208449823682309.plus.aac.p.m4a"}],"releaseDate":"2013-01-15","trackNumber":1,"url":"https://music.apple.com/us/album/duchess-for-nothing/575329457?i=575329663"},"href":"/v1/catalog/us/songs/575329663","id":"575329663","relationships":{"albums":{"data":[{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"f1f0ee","height":1400,"textColor1":"000a13","textColor2":"000c11","textColor3":"30383e","textColor4":"30393d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/ec/33/ed/ec33ed1a-0cf5-1478-41eb-51ce063f92d6/Cover.jpg/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2012 HHBTM","genreNames":["Alternative","Music","Rock","College Rock","Pop","Pop/Rock","Indie Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Genius Fatigue","playParams":{"id":"575329457","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2013-01-15","trackCount":10,"upc":"795103607620","url":"https://music.apple.com/us/album/genius-fatigue/575329457"},"href":"/v1/catalog/us/albums/575329457","id":"575329457","type":"albums"}],"href":"/v1/catalog/us/songs/575329663/albums"},"artists":{"data":[{"attributes":{"artwork":{"bgColor":"bc0068","height":620,"textColor1":"ffffff","textColor2":"f5e7ef","textColor3":"f1cbe0","textColor4":"eab9d4","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/93/a3/fe/mzi.nmmldnsd.jpg/{w}x{h}ac.jpg","width":620},"genreNames":["Alternative"],"name":"TunaBunny","url":"https://music.apple.com/us/artist/tunabunny/333943109"},"href":"/v1/catalog/us/artists/333943109","id":"333943109","relationships":{"albums":{"data":[{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"ee993f","height":1500,"textColor1":"0a0907","textColor2":"191712","textColor3":"372612","textColor4":"43311b","url":"https://is1-ssl.mzstatic.com/image/thumb/Music111/v4/3e/ef/e8/3eefe81a-c8fb-70db-a7ea-a54bebdcfa46/191018833985.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2017 HHBTM Records","genreNames":["Rock","Music","Alternative","College Rock","Electronic","Pop","Pop/Rock","Indie Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Pcp Presents Alice in Wonderland Jr.","playParams":{"id":"1205767123","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2017-06-09","trackCount":28,"upc":"191018833985","url":"https://music.apple.com/us/album/pcp-presents-alice-in-wonderland-jr/1205767123"},"href":"/v1/catalog/us/albums/1205767123","id":"1205767123","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"bc0068","height":620,"textColor1":"ffffff","textColor2":"f5e7ef","textColor3":"f1cbe0","textColor4":"eab9d4","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/93/a3/fe/mzi.nmmldnsd.jpg/{w}x{h}bb.jpg","width":620},"contentRating":"clean","copyright":"℗ 2011 HHBTM","genreNames":["Indie Rock","Music","Alternative","College Rock","Rock","Pop","Pop/Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Minima Moralia","playParams":{"id":"464244644","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2011-09-27","trackCount":12,"upc":"847108031723","url":"https://music.apple.com/us/album/minima-moralia/464244644"},"href":"/v1/catalog/us/albums/464244644","id":"464244644","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"f1f0ee","height":1400,"textColor1":"000a13","textColor2":"000c11","textColor3":"30383e","textColor4":"30393d","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/v4/ec/33/ed/ec33ed1a-0cf5-1478-41eb-51ce063f92d6/Cover.jpg/{w}x{h}bb.jpg","width":1400},"copyright":"℗ 2012 HHBTM","genreNames":["Alternative","Music","Rock","College Rock","Pop","Pop/Rock","Indie Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Genius Fatigue","playParams":{"id":"575329457","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2013-01-15","trackCount":10,"upc":"795103607620","url":"https://music.apple.com/us/album/genius-fatigue/575329457"},"href":"/v1/catalog/us/albums/575329457","id":"575329457","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"ffffff","height":620,"textColor1":"000000","textColor2":"161616","textColor3":"333333","textColor4":"454545","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/70/10/bf/mzi.tkuuzput.jpg/{w}x{h}bb.jpg","width":620},"copyright":"℗ 2010 Happy Happy Birthday to Me Records","genreNames":["Indie Rock","Music","Alternative","College Rock","Pop","Pop/Rock","Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Tunabunny","playParams":{"id":"377527201","kind":"album"},"recordLabel":"Happy Happy Birthday to Me Records","releaseDate":"2010-07-20","trackCount":15,"upc":"844185050173","url":"https://music.apple.com/us/album/tunabunny/377527201"},"href":"/v1/catalog/us/albums/377527201","id":"377527201","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"f48a21","height":1500,"textColor1":"090404","textColor2":"400c0c","textColor3":"381f0a","textColor4":"642510","url":"https://is1-ssl.mzstatic.com/image/thumb/Music4/v4/03/a7/e1/03a7e1fd-e31e-1a1a-1ca5-a6845953886c/888608550956.jpg/{w}x{h}bb.jpg","width":1500},"copyright":"℗ 2014 HHBTM Records","genreNames":["Alternative","Music","Electronic","Pop","Pop/Rock","Rock"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":false,"name":"Kingdom Technology","playParams":{"id":"818880961","kind":"album"},"recordLabel":"HHBTM","releaseDate":"2014-03-11","trackCount":14,"upc":"888608550956","url":"https://music.apple.com/us/album/kingdom-technology/818880961"},"href":"/v1/catalog/us/albums/818880961","id":"818880961","type":"albums"},{"attributes":{"artistName":"TunaBunny","artwork":{"bgColor":"c7c2bc","height":620,"textColor1":"000f30","textColor2":"111b33","textColor3":"28334c","textColor4":"353c4f","url":"https://is1-ssl.mzstatic.com/image/thumb/Music/62/72/0f/mzi.etiucsro.jpg/{w}x{h}bb.jpg","width":620},"copyright":"℗ 2009 Happy Happy Birthday to Me Records","genreNames":["Alternative","Music"],"isCompilation":false,"isComplete":true,"isMasteredForItunes":false,"isSingle":true,"name":"Outerspace Is the Center of the Earth - Single","playParams":{"id":"333943019","kind":"album"},"recordLabel":"Happy Happy Birthday to Me Records","releaseDate":"2009-09-15","trackCount":1,"upc":"844185068789","url":"https://music.apple.com/us/album/outerspace-is-the-center-of-the-earth-single/333943019"},"href":"/v1/catalog/us/albums/333943019","id":"333943019","type":"albums"}],"href":"/v1/catalog/us/artists/333943109/albums"}},"type":"artists"}],"href":"/v1/catalog/us/songs/575329663/artists"}},"type":"songs"}"#).unwrap())
    }
}
