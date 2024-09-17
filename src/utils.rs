use crate::{
    error::{Error, Result},
    track::Track,
};
use id3::TagLike;
use reqwest::Client;
// use std::io::{Read, Write};

pub async fn add_metadata_to_mp3(
    client: &Client,
    mp3_file_path: &str,
    track: &Track,
    overwrite_artwork: bool,
) -> Result<()> {
    let mut tag = match id3::Tag::read_from_path(mp3_file_path) {
        Ok(tag) => tag,
        Err(e) => match e.kind {
            id3::ErrorKind::NoTag => id3::Tag::new(),
            _ => return Err(Error::TagError),
        },
    };

    tag.set_title(track.name.to_owned());
    tag.set_album(track.album.to_owned());
    tag.set_artist(track.artists.get(0).ok_or(Error::MalformedTrackError)?);
    tag.set_year(track.release_year as i32);

    if overwrite_artwork == false {
        if let Some(_) = tag.pictures().next() {
            eprintln!("{} already has an image", mp3_file_path);
            match tag.write_to_path(mp3_file_path, tag.version()) {
                Ok(..) => return Ok(()),
                Err(..) => return Err(Error::TagError),
            }
        }
    }

    if let Ok(bytes) = download_best_artwork_bytes(client, track).await {
        tag.remove_all_pictures();

        tag.add_frame(id3::frame::Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: id3::frame::PictureType::CoverFront,
            description: "cover art".to_string(),
            data: bytes,
        });
    }

    match tag.write_to_path(mp3_file_path, tag.version()) {
        Ok(..) => Ok(()),
        Err(..) => Err(Error::TagError),
    }
}

pub async fn add_metadata_to_m4a(
    client: &Client,
    m4a_file_path: &str,
    track: &Track,
    overwrite_artwork: bool,
) -> Result<()> {
    let mut tag = match mp4ameta::Tag::read_from_path(m4a_file_path) {
        Ok(tag) => tag,
        Err(..) => return Err(Error::TagError),
    };

    tag.set_title(track.name.to_owned());
    tag.set_album(track.album.to_owned());
    tag.set_artist(track.artists.get(0).ok_or(Error::MalformedTrackError)?);
    tag.set_year(track.release_year.to_string());

    if overwrite_artwork == false {
        if let Some(_) = tag.images().next() {
            eprintln!("{} already has an image", m4a_file_path);
            match tag.write_to_path(m4a_file_path) {
                Ok(..) => return Ok(()),
                Err(..) => return Err(Error::TagError),
            }
        }
    }

    if let Ok(bytes) = download_best_artwork_bytes(client, track).await {
        tag.add_artwork(mp4ameta::Img::new(mp4ameta::ImgFmt::Jpeg, bytes));
    }

    match tag.write_to_path(m4a_file_path) {
        Ok(..) => Ok(()),
        Err(..) => Err(Error::TagError),
    }
}

async fn download_best_artwork_bytes(client: &Client, track: &Track) -> Result<Vec<u8>> {
    let mut token: Option<&str> = None;
    let mut artwork_url: Option<String> = None;

    if let Some(apple_music) = &track.services.apple_music {
        if let Some(url) = &apple_music.image {
            token = Some(crate::apple_music::AppleMusic::PUBLIC_BEARER_TOKEN);
            artwork_url = Some(url.replace(".webp", ".jpg"));
        }
    } else if let Some(spotify) = &track.services.spotify {
        if let Some(url) = &spotify.image {
            artwork_url = Some(url.to_owned());
        }
    }

    if let Some(url) = artwork_url {
        let response = match token {
            Some(token) => client.get(url).bearer_auth(token),
            None => client.get(url),
        }
        .send()
        .await?
        .error_for_status()?;

        Ok(response.bytes().await?.to_vec())
    } else {
        Err(Error::TagError)
    }
}
