// // use anyhow::{Context, Result};
// use clap::{ArgGroup, Parser};
// use std::path::PathBuf;

// mod apple_music;
// mod bandcamp;
// mod service;
// mod spotify;
// mod track;
// mod youtube;

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
