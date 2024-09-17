// // use anyhow::{Context, Result};
// use clap::{ArgGroup, Parser};
// use std::path::PathBuf;
//

mod apple_music;
mod bandcamp;
mod error;
mod playlist;
mod service;
mod spotify;
mod track;
mod utils;
mod youtube;

use apple_music::AppleMusic;
use bandcamp::Bandcamp;
use error::{Error, Result};
use playlist::Playlist;
use reqwest::Client;
use service::{Album, Artist, Services};
use spotify::{SessionInfo, Spotify};
use std::io::Write;
use track::Track;
use youtube::{Payload, YouTube};

fn spotify_playlist_conversion_sanity_check(playlist: &Playlist) -> Result<()> {
    let mut youtube_miss_count = 0;
    let mut apple_music_miss_count = 0;
    let mut bandcamp_miss_count = 0;
    println!(
        "Sanity checking {} tracks for playlist {}:",
        playlist.tracks.len(),
        playlist.name
    );
    let mut count = 1;
    for track in &playlist.tracks {
        println!("{}:\t{} - {}", count, &track.name, &track.artists[0]);
        match track.services.youtube {
            Some(..) => (),
            None => {
                println!("\t\t*no YouTube");
                youtube_miss_count += 1;
            }
        }
        match track.services.apple_music {
            Some(..) => (),
            None => {
                println!("\t\t*no Apple Music");
                apple_music_miss_count += 1;
            }
        }
        match track.services.bandcamp {
            Some(..) => (),
            None => {
                println!("\t\t*no Bandcamp");
                bandcamp_miss_count += 1;
            }
        }
        count += 1;
    }

    println!("\n{} tracks without YouTube links", youtube_miss_count);
    println!(
        "{} tracks without Apple Music links",
        apple_music_miss_count
    );
    println!("{} tracks without Bandcamp links", bandcamp_miss_count);

    Ok(())
}

async fn show_downloader(
    client: &Client,
    spotify_id: &str,
    filename: &str,
    download_tracks: bool,
) -> Result<Playlist> {
    let start = std::time::Instant::now();

    let session_info: SessionInfo = Spotify::get_public_session_info(&client).await?;

    let playlist_file_path = "./example_show_playlists/";
    let playlist_download_path = "./example_show_playlists/downloads/";

    let playlist: Playlist =
        match Playlist::from_file(&format!("{}{}.json", playlist_file_path, filename)) {
            Ok(p) => {
                println!(
                    "Playlist already exists. Loading {}",
                    &format!("{}{}.json", playlist_file_path, filename)
                );
                p
            }
            Err(..) => {
                let mut playlist = Playlist::from_spotify_id(
                    &client,
                    &session_info.access_token,
                    spotify_id,
                    &filename.to_string(),
                    playlist_file_path,
                )
                .await?;
                playlist
                    .add_apple_music(&client, AppleMusic::PUBLIC_BEARER_TOKEN)
                    .await?;
                playlist.add_bandcamp(&client).await?;
                playlist.add_youtube_(&client).await?;

                playlist.save_to_file(playlist_file_path, &filename.to_string())?;

                playlist
            }
        };

    let middle = std::time::Instant::now();

    println!(
        "Playlist Match/Load took {} seconds",
        (middle - start).as_secs()
    );

    println!("");
    spotify_playlist_conversion_sanity_check(&playlist)?;

    if download_tracks == true {
        println!("");
        playlist
            .download_tracks(
                &client,
                &format!("{}{}/", playlist_download_path, filename),
                true,
            )
            .await?;
    }

    let end = std::time::Instant::now();

    println!(
        "Playlist Download took {} seconds",
        (end - middle).as_secs()
    );

    println!("");
    println!("Total time: {} seconds", (end - start).as_secs());
    println!("\n----------\nEnd.");

    Ok(playlist)
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let client: Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;

    let test_oct_15_2023_playlist_id = "6lJfNdZUnat3Qa4kKaVIxG";
    let filename = "tester_oct_15_2023";

    let playlist = show_downloader(&client, test_oct_15_2023_playlist_id, filename, true).await?;

    // let image_bytes = utils::download_image_bytes(
    //     &client,
    //     // "https://www.rd.com/wp-content/uploads/2021/01/GettyImages-1175550351.jpg?resize=2048",
    //     "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fimg.freepik.com%2Fpremium-photo%2Flifelike-cheese-squares-resembling_1002329-375.jpg&f=1&nofb=1&ipt=f04db5d61ae98b00867aab0128ccf07898156b32afe99017764c623fc0c10c32&ipo=images",
    //     Some(AppleMusic::PUBLIC_BEARER_TOKEN),
    // )
    // .await?;

    // playlist.tracks[0]
    //     .download(&client, "./", "cheese", true)
    //     .await?;

    // utils::add_artwork_to_mp3(image_bytes, &x, true)?;
    // println!("{}", x);
    // let mut tracks: Vec<Track> = serde_json::from_str(&std::fs::read_to_string("./test.json")?)?;
    // let track: &mut Track = &mut tracks[3];

    // match track
    //     .services
    //     .youtube
    //     .as_ref()
    //     .ok_or(Error::DownloadError)?
    //     .download(
    //         &client,
    //         &format!("{} - {}", &track.name, &track.artists.join(", ")),
    //         "./test_downloads/",
    //     )
    //     .await
    // {
    //     Ok(..) => println!("done."),
    //     Err(e) => println!("{}", e),
    // }

    Ok(())
}
