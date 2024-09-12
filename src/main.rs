// // use anyhow::{Context, Result};
// use clap::{ArgGroup, Parser};
// use std::path::PathBuf;
//

mod apple_music;
mod bandcamp;
mod error;
mod service;
mod spotify;
mod track;
mod youtube;

use apple_music::AppleMusic;
use bandcamp::Bandcamp;
use error::{Error, Result};
use reqwest::Client;
use service::{Album, Artist, Services};
use spotify::{SessionInfo, Spotify};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use track::Track;
use youtube::{Payload, YouTube};

// #[derive(Parser)]
// #[command(version, about, long_about = None)]
// #[clap(group(
//         ArgGroup::new("spotify-data-type")
//         .required(true)
//         .args(&["playlist", "album", "track"]),
//     ))]
// struct Cli {
//     /// Spotify playlist URL
//     #[arg(short, long, value_name = "URL")]
//     playlist: Option<String>,

//     /// Spotify album URL
//     #[arg(short, long, value_name = "URL")]
//     album: Option<String>,

//     /// Spotify track URL
//     #[arg(short, long, value_name = "URL")]
//     track: Option<String>,

//     /// Directory to put downloaded track(s)
//     #[arg(short, long, value_name = "DIR")]
//     download: Option<PathBuf>,

//     /// Directory to put metadata file(s)
//     #[arg(short, long, value_name = "DIR")]
//     metadata: Option<PathBuf>,
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let cli = Cli::parse();

//     if let Some(download_path) = cli.download.as_deref() {
//         println!("Download Path: {}", download_path.display());
//     }

//     if let Some(md_path) = cli.metadata.as_deref() {
//         println!("Metadata Path: {}", md_path.display());
//     }

//     if let Some(playlist) = cli.playlist.as_deref() {
//         println!("Playlist URL: {}", playlist);
//     } else if let Some(album) = cli.album.as_deref() {
//         println!("Album URL: {}", album);
//     } else if let Some(track) = cli.track.as_deref() {
//         println!("Track URL: {}", track);
//     }

//     Ok(())
// }
//
async fn convert_spotify_playlist(
    client: &Client,
    spotify_auth: &str,
    spotify_playlist_id: &str,
) -> Result<Vec<Track>> {
    let mut tracks: Vec<Track> =
        Spotify::create_playlist_from_id(client, spotify_auth, spotify_playlist_id).await?;

    let num_tracks = tracks.len();

    let mut apple_music_service_futures = Vec::with_capacity(num_tracks);
    for track in &mut tracks {
        apple_music_service_futures
            .push(track.add_apple_music(AppleMusic::PUBLIC_BEARER_TOKEN, client));
    }
    let results = futures::future::join_all(apple_music_service_futures).await;
    let mut count = 0;
    for result in results {
        match result {
            Ok(..) => (),
            Err(e) => {
                println!(
                    "Skipping adding Apple Music to track {} due to error:\n{}",
                    count + 1,
                    e
                )
            }
        };
        count += 1;
    }

    let mut youtube_service_futures = Vec::with_capacity(num_tracks);
    for track in &mut tracks {
        youtube_service_futures.push(track.add_youtube(client));
    }
    let results = futures::future::join_all(youtube_service_futures).await;
    count = 0;
    for result in results {
        match result {
            Ok(..) => (),
            Err(e) => {
                println!(
                    "Skipping adding YouTube to track {} due to error:\n{}",
                    count, e
                )
            }
        };
        count += 1;
    }

    let mut bandcamp_service_futures = Vec::with_capacity(num_tracks);
    for track in &mut tracks {
        bandcamp_service_futures.push(track.add_bandcamp(client));
    }
    let results = futures::future::join_all(bandcamp_service_futures).await;
    count = 0;
    for result in results {
        match result {
            Ok(..) => (),
            Err(e) => {
                println!(
                    "Skipping adding Bandcamp to track {} due to error:\n{}",
                    count, e
                )
            }
        };
        count += 1;
    }

    Ok(tracks)
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let client: Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;
    // let session_info: SessionInfo = Spotify::get_public_session_info(&client).await?;

    // println!("spotify token:\n{}", &session_info.access_token);

    // println!("\n----------\nStart:");

    // // let playlist_id: &str = "0iR3Srlz854Uw0Ci4gd4HL"; // 68: sept 01 2024
    // let playlist_id = "0L4V9lrwrSP88sxIxz2eJT"; // 69: sep 08 2024
    // let tracks = convert_spotify_playlist(&client, &session_info.access_token, playlist_id).await?;
    // let mut youtube_miss_count = 0;
    // let mut apple_music_miss_count = 0;
    // let mut bandcamp_miss_count = 0;
    // println!("\n{} tracks:", tracks.len());
    // let mut count = 0;
    // for track in &tracks {
    //     println!("{}: {} - {}", count, &track.name, &track.artists[0]);
    //     match track.services.youtube {
    //         Some(..) => (),
    //         None => {
    //             println!("\t* no YouTube");
    //             youtube_miss_count += 1;
    //         }
    //     }
    //     match track.services.apple_music {
    //         Some(..) => (),
    //         None => {
    //             println!("\t* no Apple Music");
    //             apple_music_miss_count += 1;
    //         }
    //     }
    //     match track.services.bandcamp {
    //         Some(..) => (),
    //         None => {
    //             println!("\t* no Bandcamp");
    //             bandcamp_miss_count += 1;
    //         }
    //     }
    //     count += 1;
    // }
    // println!("");
    // println!("{} tracks without YouTube links", youtube_miss_count);
    // println!(
    //     "{} tracks without Apple Music links",
    //     apple_music_miss_count
    // );
    // println!("{} tracks without Bandcamp links", bandcamp_miss_count);

    // let mut file = std::fs::File::create("./test.json")?;
    // file.write_all(serde_json::to_string_pretty(&tracks)?.as_bytes())?;

    // println!("\n----------\nDone.");

    let mut tracks: Vec<Track> = serde_json::from_str(&std::fs::read_to_string("./test.json")?)?;
    let track: &mut Track = &mut tracks[0];

    match track
        .services
        .bandcamp
        .as_ref()
        .ok_or(Error::DownloadError)?
        .download(&client, &track.name, "./test_downloads")
        .await
    {
        Ok(..) => println!("done."),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
