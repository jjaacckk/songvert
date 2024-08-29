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

use crate::error::{Error, Result};
use crate::service::{Album, Artist, Services};
use crate::spotify::{SessionInfo, Spotify};
use crate::track::Track;

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
#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let client: reqwest::Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()
            .unwrap();
    let session_info: SessionInfo = Spotify::get_public_session_info(&client).await.unwrap();

    println!("token: {}", &session_info.access_token);

    let spotify_track: Track = match Spotify::create_track_from_id(
        &client,
        &session_info.access_token,
        "6K225HZ3V7F4ec7yi1o88C",
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            return Err(e);
        }
    };

    println!("{:?}", spotify_track);

    // let apple_music_track: Track = match AppleMusic::create_track_from_id(
    //     &client,
    //     AppleMusic::PUBLIC_BEARER_TOKEN,
    //     "575329663",
    // )
    // .await
    // {
    //     Ok(t) => t,
    //     Err(e) => {
    //         eprintln!("{}", e);
    //         return Err(e);
    //     }
    // };

    // // println!("{:?}", apple_music_track);

    // println!(
    //     "Apple Music\t\t|\t\tSpotify\n{}\t\t|\t\t{}\n{}\t\t|\t\t{}\n{}\t\t|\t\t{}",
    //     apple_music_track.name,
    //     spotify_track.name,
    //     apple_music_track.artists.join(", "),
    //     spotify_track.artists.join(", "),
    //     apple_music_track.album,
    //     spotify_track.album
    // );

    // let track = AppleMusic::get_raw(
    //     &client,
    //     AppleMusic::PUBLIC_BEARER_TOKEN,
    //     "catalog/us/songs?filter[isrc]=USZUD1215001&include=albums,artists",
    // )
    // .await
    // .unwrap();

    // match AppleMusic::get_raw_track_match_from_track(
    //     &client,
    //     AppleMusic::PUBLIC_BEARER_TOKEN,
    //     &apple_music_track,
    // )
    // .await
    // {
    //     Ok(t) => {
    //         // println!("name: {}", t["attributes"]["name"]);
    //         println!("{}", t);
    //     }
    //     Err(e) => {
    //         eprintln!("{}", e);
    //         return Err(e);
    //     }
    // };

    // let search_result: serde_json::Value = AppleMusic::get_raw(
    //     &client,
    //     &AppleMusic::PUBLIC_BEARER_TOKEN,
    //     &format!(
    //         "catalog/us/songs?filter[isrc]={}&include=albums,artists",
    //         "USZUD1215001"
    //     ),
    // )
    // .await
    // .unwrap()["data"][0]
    //     .to_owned();
    //

    // let example_spotify_service: Spotify = Spotify {
    //     id: String::from("6K225HZ3V7F4ec7yi1o88C"),
    //     artists: vec![Artist {id: String::from("0xiwsYZwhrizQGNaQtW942"), name: String::from("Tunabunny")}],
    //     album: Album { id: String::from("6WSL47W7Z5WwCCKzaFyLGd"), name: String::from("Genius Fatigue"), total_tracks: 10, ean: None, upc: None},
    //     url: String::from("https://open.spotify.com/track/6K225HZ3V7F4ec7yi1o88C"),
    //     image: Some(String::from("https://i.scdn.co/image/ab67616d0000b27336a71c545ed453f80433f6c8")),
    //     audio_preview: Some(String::from("https://p.scdn.co/mp3-preview/13a7bfeabbe56d852fb9f7b6291c7dc49bcde515?cid=d8a5ed958d274c2e8ee717e6a4b0971d")),
    // };

    Ok(())
}
