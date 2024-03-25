use anyhow::{Context, Result};
use clap::{ArgGroup, Parser, Subcommand};
use std::path::PathBuf;

// #[derive(Parser, Debug)]
// #[command(version, about)]
// struct Args {
//     /// Spotify playlist URL
//     #[arg(short, long)]
//     playlist: Option<String>,

//     /// Spotify album URL
//     #[arg(short, long)]
//     album: Option<String>,

//     /// Spotify track URL
//     #[arg(short, long)]
//     track: Option<String>,

//     /// Number of times to greet
//     #[arg(short, long, default_value_t = 1)]
//     count: u8,
// }

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[clap(group(
        ArgGroup::new("spotify-data-type")
        .required(true)
        .args(&["playlist", "album", "track"]),
    ))]
// #[clap(group(
//         ArgGroup::new("spotify-download-args")
//         .required(true)
//         .args(&["playlist", "album", "track"]),
//     ))]
struct Cli {
    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
    ////
    // /// Spotify URL
    // url: String,
    /// Spotify playlist URL
    #[arg(short, long, value_name = "URL")]
    playlist: Option<String>,

    /// Spotify album URL
    #[arg(short, long, value_name = "URL")]
    album: Option<String>,

    /// Spotify track URL
    #[arg(short, long, value_name = "URL")]
    track: Option<String>,

    /// Directory to put downloaded track(s)
    #[arg(short, long, value_name = "DIR")]
    download: Option<PathBuf>,

    /// Directory to put metadata file(s)
    #[arg(short, long, value_name = "DIR")]
    metadata: Option<PathBuf>,
    ////
    // #[command(subcommand)]
    // command: Option<Commands>,
}

// #[derive(Subcommand)]
// enum Commands {
//     /// Download Spotify track(s) and metadata
//     #[clap(group(
//     ArgGroup::new("spotify-data-type")
//     .required(true)
//     .args(&["playlist", "album", "track"]),
// ))]
//     Download {
//         /// Spotify URL
//         url: String,

//         /// Spotify playlist URL
//         #[arg(short, long, action)]
//         playlist: bool,

//         /// Spotify album URL
//         #[arg(short, long, action)]
//         album: bool,

//         /// Spotify track URL
//         #[arg(short, long, action)]
//         track: bool,
//     },
// }

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(download_path) = cli.download.as_deref() {
        println!("Download Path: {}", download_path.display());
    }

    if let Some(md_path) = cli.metadata.as_deref() {
        println!("Metadata Path: {}", md_path.display());
    }

    if let Some(playlist) = cli.playlist.as_deref() {
        println!("Playlist URL: {}", playlist);
    } else if let Some(album) = cli.album.as_deref() {
        println!("Album URL: {}", album);
    } else if let Some(track) = cli.track.as_deref() {
        println!("Track URL: {}", track);
    }

    // match &cli.command {
    //     Some(Commands::Download {
    //         url,
    //         playlist,
    //         album,
    //         track,
    //     }) => {
    //         println!("URL: {}", url);

    //         if *playlist {
    //             println!("This is a playlist");
    //         }

    //         if *album {
    //             println!("This is an album");
    //         }

    //         if *track {
    //             println!("This is a track");
    //         }
    //     }

    //     // Some(Commands::Album { url }) => {
    //     //     println!("Album URL: {}", url);
    //     // }
    //     // Some(Commands::Track { url }) => {
    //     //     println!("Track URL: {}", url);
    //     // }
    //     None => {}
    // }

    Ok(())
}
