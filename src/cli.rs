use clap::{Args, Parser};
use reqwest::Client;
use songvert::{
    apple_music::*, bandcamp::*, error::*, playlist::*, service::*, spotify::*, track::*,
    youtube::*,
};
use std::path::{Path, PathBuf};

/// Easily convert URLs between Spotify, Apple Music, Bandcamp, and YouTube
#[derive(Parser, Debug)]
#[command(name = "Songvert", version)]
pub struct Cli {
    #[command(flatten)]
    input: SourceInput,

    /// Indicates input is FILE (not URL)
    #[arg(short, long)]
    file: bool,

    #[command(flatten)]
    conversion_outputs: ConversionServices,

    /// Download Track(s) to directory
    #[arg(short, long)]
    download_directory: Option<PathBuf>,

    /// Save all JSON metadata to file
    #[arg(short, long, value_name = "FILE")]
    output_file: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct SourceInput {
    /// Album URL (pass -f for FILE)
    #[arg(short, long)]
    album: Option<String>,

    /// Playlist URL (pass -f for FILE)
    #[arg(short, long)]
    playlist: Option<String>,

    /// Track URL (pass -f for FILE)
    #[arg(short, long)]
    track: Option<String>,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = true)]
struct ConversionServices {
    /// Output Spotify URL(s)
    #[arg(short = 'S', long)]
    spotify: bool,

    /// Output Apple Music URL(s)
    #[arg(short = 'A', long)]
    apple_music: bool,

    /// Output Bandcamp URL(s)
    #[arg(short = 'B', long)]
    bandcamp: bool,

    /// Output YouTube URL(s)
    #[arg(short = 'Y', long)]
    youtube: bool,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;

        if let Some(_album) = &self.input.album {
            return Err(Error::TrackError(
                "Album functionality has not been implemented".to_string(),
            ));
        } else if let Some(playlist_str) = &self.input.playlist {
            let mut playlist = {
                if self.file {
                    let playlist_path = PathBuf::from(playlist_str);
                    Playlist::from_file(&playlist_path)?
                } else {
                    let source_info = get_source_info_from_url(&playlist_str)?;
                    match source_info.service {
                        Source::Spotify => {
                            let session_info = Spotify::get_public_session_info(&client).await?;
                            Playlist::from_spotify_id().await?
                        }
                        Source::AppleMusic => Playlist::from_apple_music_id().await?,
                    }
                }
            };

            if self.conversion_outputs.spotify && playlist.source_service != Source::Spotify {
                let session_info = Spotify::get_public_session_info(&client).await?;
                playlist
                    .add_spotify(&client, &session_info.access_token)
                    .await?;
            }

            if self.conversion_outputs.apple_music && playlist.source_service != Source::AppleMusic
            {
                playlist
                    .add_apple_music(&client, AppleMusic::PUBLIC_BEARER_TOKEN)
                    .await?;
            }
            if self.conversion_outputs.bandcamp {
                playlist.add_bandcamp(&client).await?;
            }
            if self.conversion_outputs.youtube {
                playlist.add_youtube(&client).await?;
            }

            if let Some(output_path) = &self.output_file {
                playlist.save_to_file(output_path, &playlist.name)?;
            }

            if let Some(dir) = &self.download_directory {
                playlist.download_tracks(&client, dir, true).await?;
            }
        } else if let Some(track) = &self.input.track {
        }

        Ok(())
    }
}

struct SourceInfo {
    id: String,
    service: Source,
}

fn get_source_info_from_url(url: &str) -> Result<SourceInfo> {
    todo!()
}
