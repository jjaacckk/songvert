//mod cli;

//use crate::cli::Cli;
//use clap::Parser;
use reqwest::Client;
use songvert::{
    apple_music::AppleMusic,
    error::Result,
    playlist::Playlist,
    spotify::{SessionInfo, Spotify},
};
use std::path::PathBuf;

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
    let playlist_file_path = PathBuf::from("./example_show_playlists/");
    let playlist_download_path = PathBuf::from("./example_show_playlists/downloads/");

    let mut playlist_full_path_for_check = playlist_file_path.clone();
    playlist_full_path_for_check.push(filename);
    playlist_full_path_for_check.set_extension("json");

    let playlist: Playlist = match Playlist::from_file(&playlist_full_path_for_check) {
        Ok(p) => {
            println!(
                "Playlist already exists. Loading {}",
                playlist_full_path_for_check.to_string_lossy()
            );
            p
        }
        Err(..) => {
            let mut playlist = Playlist::from_spotify_id(
                &client,
                &session_info.access_token,
                spotify_id,
                &playlist_file_path,
                &filename.to_string(),
            )
            .await?;
            println!("Attempting to match {} tracks:", playlist.tracks.len());
            playlist
                .add_apple_music(&client, AppleMusic::PUBLIC_BEARER_TOKEN)
                .await?;
            playlist.add_bandcamp(&client).await?;
            playlist.add_youtube(&client).await?;

            playlist.save_to_file(&playlist_file_path, filename)?;

            playlist
        }
    };

    let middle = std::time::Instant::now();

    println!(
        "Playlist Match/Load took {} seconds ({} ms)",
        (middle - start).as_secs(),
        (middle - start).as_millis()
    );

    println!("");
    spotify_playlist_conversion_sanity_check(&playlist)?;

    if download_tracks == true {
        let mut download_path = playlist_download_path;
        download_path.push(filename);
        println!("");
        playlist
            .download_tracks(&client, &download_path, true)
            .await?;
    }

    let end = std::time::Instant::now();

    if download_tracks == true {
        println!(
            "Playlist Download took {} seconds  ({} ms)",
            (end - middle).as_secs(),
            (end - middle).as_millis()
        );
    }

    println!("");
    println!(
        "Total time: {} seconds ({} ms)",
        (end - start).as_secs(),
        (end - start).as_millis()
    );
    println!("\n----------\nEnd.");

    Ok(playlist)
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    //let cli = Cli::parse();
    //cli.run().await?;

    //println!(
    //    "{}",
    //    strsim::jaro_winkler(
    //        "It’s Good to Be Back",
    //        "It's good to be back - Metronomy x Panic Shack",
    //    )
    //);
    //
    //println!(
    //    "{}",
    //    strsim::jaro_winkler(
    //        "It's good to be back - Metronomy x Panic Shack",
    //        "It’s Good to Be Back",
    //    )
    //);

    let client: Client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;

    let playlist_id = "03AOV5ofnW9RTP3KSQZSV1";
    let filename = "show_73_oct_6_2024";

    let mut playlist = show_downloader(&client, playlist_id, filename, false).await?;

    //playlist.tracks[0].add_youtube(&client).await?;
    //playlist.tracks[7].add_youtube(&client).await?;

    //playlist.tracks[15]
    //    .download(&client, "./", "cheeser", true)
    //    .await?;

    //let track = &mut playlist.tracks[15];
    //match track.add_youtube(&client).await {
    //    Ok(..) => println!("success: {}", serde_json::to_string_pretty(&track)?),
    //    Err(e) => println!("failure: {}", e),
    //}

    //playlist.tracks[18]
    //    .download(
    //        &client,
    //        "./example_show_playlists/downloads/",
    //        "cheese",
    //        true,
    //    )
    //    .await?;

    Ok(())
}
