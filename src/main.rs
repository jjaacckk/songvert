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
use error::{Error, Result};
use reqwest::Client;
use service::{Album, Artist, Services};
use spotify::{SessionInfo, Spotify};
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

    let mut count = 0;

    // let mut apple_music_track_futures = Vec::with_capacity(tracks.len());
    // for track in &mut tracks {
    //     apple_music_track_futures
    //         .push(track.add_apple_music(AppleMusic::PUBLIC_BEARER_TOKEN, client));
    // }
    // for future in apple_music_track_futures {
    //     match future.await {
    //         Ok(..) => (),
    //         Err(e) => {
    //             println!(
    //                 "Skipping adding Apple Music to track {} due to error:\n{}",
    //                 count + 1,
    //                 e
    //             )
    //         }
    //     };
    //     count += 1;
    // }

    // count = 0;

    // let mut youtube_track_futures = Vec::with_capacity(tracks.len());
    for track in &mut tracks {
        // youtube_track_futures.push(track.add_youtube(client));
        match track.add_youtube(client).await {
            Ok(..) => (),
            Err(e) => {
                println!(
                    "Skipping adding YouTube to track {} due to error:\n{}",
                    count, e
                )
            }
        }
        count += 1;
    }
    // for future in youtube_track_futures {
    //     match future.await {
    //         Ok(..) => (),
    //         Err(e) => {
    //             println!(
    //                 "Skipping adding YouTube to track {} due to error:\n{}",
    //                 count, e
    //             )
    //         }
    //     }
    //     count += 1;
    // }

    Ok(tracks)
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let client: Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;
    let session_info: SessionInfo = Spotify::get_public_session_info(&client).await?;

    println!("spotify token:\n{}", &session_info.access_token);

    // println!("\n----------\nStart:");

    // let playlist_id: &str = "0iR3Srlz854Uw0Ci4gd4HL"; //sept 01 2024
    // let tracks = convert_spotify_playlist(&client, &session_info.access_token, playlist_id).await?;

    // println!("{} tracks:", tracks.len());
    // let mut count = 0;
    // for track in &tracks {
    //     match track.services.youtube {
    //     Some(..) => (),
    //     None => {count+=1;
    //         println!(
    //         "\t* no YT"
    //         )},
    //     }

    //     println!("{} - {}", &track.name, &track.artists[0]);
    // }

    // println!("{} tracks without youtube links", count);

    // let mut file = std::fs::File::create("./test.json")?;
    // file.write_all(serde_json::to_string_pretty(&tracks)?.as_bytes())?;

    // println!("\n----------\nDone.");

    let mut track = Spotify::create_track_from_id(
        &client,
        &session_info.access_token,
        // "1LQGkjjLocIkLqMARHKnUp",
        // "0nBBObbcOLO5Ik6ciPfRaR"
        "1rJ19XvEgTpqwfOUdtJiSg"
    )
    .await?;

    match track.add_youtube(&client).await {
        Ok(..) => println!("{}", serde_json::to_string_pretty(&track)?),
        Err(e) => println!("{}", e),
    };

    // let example_services: Services = Services {
    //     spotify: None,
    //     apple_music: None,
    //     youtube: None,
    //     bandcamp: None,
    // };

    // let mut example_track: Track = Track {
    //     name: "Duchess for Nothing".to_owned(),
    //     album: "Genius Fatigue".to_owned(),
    //     disk_number: 1,
    //     track_number: 1,
    //     artists: Vec::from(["Tunabunny".to_owned()]),
    //     release_year: 2013,
    //     release_month: None,
    //     release_day: None,
    //     is_explicit: false,
    //     duration_ms: 138026,
    //     services: example_services,
    //     isrc: Some("USZUD1215001".to_owned()),
    // };

    // example_track
    //     .add_spotify(&session_info.access_token, &client)
    //     .await?;
    // example_track
    //     .add_apple_music(AppleMusic::PUBLIC_BEARER_TOKEN, &client)
    //     .await?;
    // example_track.add_youtube(&client).await?;

    // ------------------------------------

    // println!("{}", serde_json::to_string_pretty(&example_track)?);

    // let spotify_track: Track = match Spotify::create_track_from_id(
    //     &client,
    //     &session_info.access_token,
    //     "6K225HZ3V7F4ec7yi1o88C",
    // )
    // .await
    // {
    //     Ok(t) => t,
    //     Err(e) => {
    //         eprintln!("{}", e);
    //         return Err(e);
    //     }
    // };

    // println!("{:?}", spotify_track);

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
