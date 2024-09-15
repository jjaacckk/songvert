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

// async fn convert_spotify_playlist(
//     client: &Client,
//     spotify_auth: &str,
//     spotify_playlist_id: &str,
//     playlist_filename: &str,
//     playlist_file_path: &str,
// ) -> Result<Playlist> {
//     match std::fs::read_to_string(format!("{}{}.json", playlist_file_path, playlist_filename)) {
//         Ok(playlist_string) => {
//             println!(
//                 "Playlist already downloaded\nImporting {}{}.json",
//                 playlist_file_path, playlist_filename
//             );
//             return Ok(serde_json::from_str(&playlist_string)?);
//         }
//         Err(..) => (),
//     };

//     match std::fs::create_dir_all(playlist_file_path) {
//         Ok(..) => (),
//         Err(e) => {
//             return Err(Error::IoError(e));
//         }
//     };

//     let mut playlist: Playlist =
//         Spotify::create_playlist_from_id(client, spotify_auth, spotify_playlist_id).await?;

//     let num_tracks = playlist.tracks.len();

//     println!(
//         "Attempting to match {} tracks for playlist: {}",
//         num_tracks, playlist.name
//     );

//     let mut apple_music_service_futures = Vec::with_capacity(num_tracks);
//     for track in &mut playlist.tracks {
//         apple_music_service_futures
//             .push(track.add_apple_music(AppleMusic::PUBLIC_BEARER_TOKEN, client));
//     }
//     let results = futures::future::join_all(apple_music_service_futures).await;
//     let mut count = 1;
//     for result in results {
//         match result {
//             Ok(..) => (),
//             Err(e) => {
//                 println!(
//                     "\tSkipping adding Apple Music to track {}: {}",
//                     count + 1,
//                     e
//                 )
//             }
//         };
//         count += 1;
//     }

//     let mut youtube_service_futures = Vec::with_capacity(num_tracks);
//     for track in &mut playlist.tracks {
//         youtube_service_futures.push(track.add_youtube(client));
//     }
//     let results = futures::future::join_all(youtube_service_futures).await;
//     count = 1;
//     for result in results {
//         match result {
//             Ok(..) => (),
//             Err(e) => {
//                 println!("\tSkipping adding YouTube to track {}: {}", count, e)
//             }
//         };
//         count += 1;
//     }

//     let mut bandcamp_service_futures = Vec::with_capacity(num_tracks);
//     for track in &mut playlist.tracks {
//         bandcamp_service_futures.push(track.add_bandcamp(client));
//     }
//     let results = futures::future::join_all(bandcamp_service_futures).await;
//     count = 1;
//     for result in results {
//         match result {
//             Ok(..) => (),
//             Err(e) => {
//                 println!("\tSkipping adding Bandcamp to track {}: {}", count, e)
//             }
//         };
//         count += 1;
//     }

//     println!(
//         "Attempting to save playlist data to {}{}",
//         playlist_file_path, playlist.name
//     );
//     let mut playlist_file =
//         std::fs::File::create(format!("{}{}.json", playlist_file_path, playlist_filename))?;
//     playlist_file.write_all(serde_json::to_string_pretty(&playlist)?.as_bytes())?;

//     Ok(playlist)
// }

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

// async fn download_track(client: &Client, track: &Track, path: &str, count: usize) -> bool {
//     let filename = format!("({}) {} - {}", count, track.name, track.artists.join(", "));

//     match track.download(client, &filename, path).await {
//         Ok => true,
//         Err(..) => {
//             println!(
//                 "\tSkipping downloading track {} - {}",
//                 track.name,
//                 track.artists.join(", ")
//             );
//             false
//         }
//     }
// }

// async fn download_playlist(
//     client: &Client,
//     playlist: &Playlist,
//     download_path: &str,
// ) -> Result<Vec<bool>> {
//     println!(
//         "Attempting to download {} tracks for playlist {}:",
//         playlist.tracks.len(),
//         playlist.name
//     );

//     match std::fs::create_dir_all(download_path) {
//         Ok(..) => (),
//         Err(e) => {
//             return Err(Error::IoError(e));
//         }
//     };

//     let mut count = 1;
//     let mut download_futures = Vec::with_capacity(playlist.tracks.len());
//     for track in &playlist.tracks {
//         download_futures.push(download_track(client, track, download_path, count));
//         count += 1;
//     }

//     Ok(futures::future::join_all(download_futures).await)
// }

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let client: Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;
    let session_info: SessionInfo = Spotify::get_public_session_info(&client).await?;

    // println!("spotify token:\n{}", &session_info.access_token);

    println!("\n----------\nBegin:");

    let playlist_id = "3AUn6OWbzMg6gO9GGo3FAR";
    let show_number = 0;
    let playlist_file_path = "./example_show_playlists/";
    let playlist_download_path = "./example_show_playlists/downloads/";

    let playlist: Playlist =
        match Playlist::from_file(&format!("{}{}.json", playlist_file_path, show_number)) {
            Ok(p) => {
                println!(
                    "Playlist already exists. Loading {}",
                    &format!("{}{}.json", playlist_file_path, show_number)
                );
                p
            }
            Err(..) => {
                let mut playlist = Playlist::from_spotify_id(
                    &client,
                    &session_info.access_token,
                    playlist_id,
                    &show_number.to_string(),
                    playlist_file_path,
                )
                .await?;
                playlist
                    .add_apple_music(&client, AppleMusic::PUBLIC_BEARER_TOKEN)
                    .await?;
                playlist.add_bandcamp(&client).await?;
                playlist.add_youtube_(&client).await?;

                playlist.save_to_file(playlist_file_path, &show_number.to_string())?;

                playlist
            }
        };

    println!("");
    spotify_playlist_conversion_sanity_check(&playlist)?;

    println!("");
    playlist
        .download_tracks(
            &client,
            &format!("{}{}/", playlist_download_path, show_number),
        )
        .await?;

    println!("\n----------\nEnd.");

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
