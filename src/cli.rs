use clap::{Args, Parser};
use reqwest::Client;
use songvert::{
    apple_music::*, bandcamp::*, error::*, playlist::*, service::*, spotify::*, track::*,
    youtube::*,
};
use std::path::PathBuf;

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

    /// Verbose printouts
    #[arg(short, long)]
    verbose: bool,

    /// Spotify Access Token
    #[arg(long = "ST", value_name = "TOKEN")]
    spotify_access_token: Option<PathBuf>,

    /// Apple Music Bearer Token
    #[arg(long = "AT", value_name = "TOKEN")]
    apple_music_bearer_token: Option<PathBuf>,
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
        let log_env = if self.verbose {
            env_logger::Env::default().filter_or("MY_LOG_LEVEL", "trace")
        } else {
            env_logger::Env::default().filter_or("MY_LOG_LEVEL", "info")
        };

        env_logger::init_from_env(log_env);

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .build()?;

        let spotify_access_token: Option<String> = match &self.spotify_access_token {
            Some(token) => Some(token.to_string_lossy().into_owned()),
            None => None,
        };

        let apple_music_bearer_token: &str = match &self.apple_music_bearer_token {
            Some(token) => &token.to_string_lossy(),
            None => AppleMusic::PUBLIC_BEARER_TOKEN,
        };

        if let Some(_album_str) = &self.input.album {
            todo!()
        } else if let Some(playlist_str) = &self.input.playlist {
            let mut playlist = {
                if self.file {
                    let playlist_path = PathBuf::from(playlist_str);
                    Playlist::from_file(&playlist_path)?
                } else {
                    let source_info = get_source_info_from_playlist_url(&playlist_str)?;
                    match source_info.service {
                        Source::Spotify => match &spotify_access_token {
                            Some(token) => {
                                Playlist::from_spotify_id(&client, &token, &source_info.id).await?
                            }
                            None => {
                                return Err(Error::DatabaseError(
                                    "No Spotify Access Token (--ST) Provided".to_string(),
                                ))
                            }
                        },
                        Source::AppleMusic => {
                            Playlist::from_apple_music_id(
                                &client,
                                apple_music_bearer_token,
                                &source_info.id,
                            )
                            .await?
                        }
                    }
                }
            };

            if self.conversion_outputs.spotify && playlist.source_service != Source::Spotify {
                match &spotify_access_token {
                    Some(token) => {
                        playlist.add_spotify(&client, &token).await?;
                    }
                    None => {
                        return Err(Error::DatabaseError(
                            "No Spotify Access Token (--ST) Provided".to_string(),
                        ))
                    }
                }
            }

            if self.conversion_outputs.apple_music && playlist.source_service != Source::AppleMusic
            {
                playlist
                    .add_apple_music(&client, apple_music_bearer_token)
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
            } else {
                println!("{}", serde_json::to_string_pretty(&playlist)?);
            }

            if let Some(dir) = &self.download_directory {
                playlist.download_tracks(&client, dir, true).await?;
            }
        } else if let Some(track_str) = &self.input.track {
            let mut track = {
                if self.file {
                    let track_path = PathBuf::from(track_str);
                    Track::from_file(&track_path)?
                } else {
                    let source_info = get_source_info_from_track_url(&track_str)?;
                    match source_info.service {
                        Source::Spotify => match &spotify_access_token {
                            Some(token) => {
                                Track::from_spotify_id(&client, &token, &source_info.id).await?
                            }
                            None => {
                                return Err(Error::DatabaseError(
                                    "No Spotify Access Token (--ST) Provided".to_string(),
                                ))
                            }
                        },
                        Source::AppleMusic => {
                            Track::from_apple_music_id(
                                &client,
                                apple_music_bearer_token,
                                &source_info.id,
                            )
                            .await?
                        }
                    }
                }
            };

            if self.conversion_outputs.spotify && track.source_service != Source::Spotify {
                match &spotify_access_token {
                    Some(token) => {
                        match track.add_spotify(&client, &token).await {
                            Ok(..) => (),
                            Err(e) => log::warn!("unable to add Spotify: {}", e),
                        };
                    }
                    None => {
                        return Err(Error::DatabaseError(
                            "No Spotify Access Token (--ST) Provided".to_string(),
                        ))
                    }
                }
            }

            if self.conversion_outputs.apple_music && track.source_service != Source::AppleMusic {
                match track
                    .add_apple_music(&client, apple_music_bearer_token)
                    .await
                {
                    Ok(..) => (),
                    Err(e) => log::warn!("unable to add Apple Music: {}", e),
                };
            }
            if self.conversion_outputs.bandcamp {
                match track.add_bandcamp(&client).await {
                    Ok(..) => (),
                    Err(e) => log::warn!("unable to add Bandcamp: {}", e),
                };
            }
            if self.conversion_outputs.youtube {
                match track.add_youtube(&client).await {
                    Ok(..) => (),
                    Err(e) => log::warn!("unable to add YouTube: {}", e),
                };
            }

            if let Some(output_path) = &self.output_file {
                track.save_to_file(
                    output_path,
                    &format!("{} - {}", track.name, track.artists[0]),
                )?;
            } else {
                println!("{}", serde_json::to_string_pretty(&track)?);
            }

            if let Some(dir) = &self.download_directory {
                track.download(&client, dir, &track.name, true).await?;
            }
        }

        Ok(())
    }
}

struct SourceInfo<'a> {
    id: &'a str,
    service: Source,
}

fn get_source_info_from_playlist_url(url: &str) -> Result<SourceInfo> {
    let apple_music_re =
        regex::Regex::new(r#"(?:https://)?music\.apple\.com/\S\S/playlist/\S+/(pl\.\S{32})"#)?;

    if let Some(captures) = apple_music_re.captures(url) {
        if let Some(m) = captures.get(1) {
            return Ok(SourceInfo {
                id: m.as_str(),
                service: Source::AppleMusic,
            });
        }
    }

    let spotify_re = regex::Regex::new(r#"(?:https://)?open\.spotify\.com/playlist/(\S{22})"#)?;

    if let Some(captures) = spotify_re.captures(url) {
        if let Some(m) = captures.get(1) {
            return Ok(SourceInfo {
                id: m.as_str(),
                service: Source::Spotify,
            });
        }
    }

    Err(Error::TrackError(
        "Not valid input playlist URL".to_string(),
    ))
}

fn get_source_info_from_album_url(url: &str) -> Result<SourceInfo> {
    let apple_music_re =
        regex::Regex::new(r#"(?:https://)?music\.apple\.com/\S\S/album/\S+/(\d{10})"#)?;

    if let Some(captures) = apple_music_re.captures(url) {
        if let Some(m) = captures.get(1) {
            return Ok(SourceInfo {
                id: m.as_str(),
                service: Source::AppleMusic,
            });
        }
    }

    let spotify_re = regex::Regex::new(r#"(?:https://)?open\.spotify\.com/album/(\S{22})"#)?;

    if let Some(captures) = spotify_re.captures(url) {
        if let Some(m) = captures.get(1) {
            return Ok(SourceInfo {
                id: m.as_str(),
                service: Source::Spotify,
            });
        }
    }

    Err(Error::TrackError("Not valid input album URL".to_string()))
}
fn get_source_info_from_track_url(url: &str) -> Result<SourceInfo> {
    let apple_music_re =
        regex::Regex::new(r#"(?:https://)?music\.apple\.com/\S\S/album/\S+/\d{10}\?i=(\d{10})"#)?;

    if let Some(captures) = apple_music_re.captures(url) {
        if let Some(m) = captures.get(1) {
            return Ok(SourceInfo {
                id: m.as_str(),
                service: Source::AppleMusic,
            });
        }
    }

    let spotify_re = regex::Regex::new(r#"(?:https://)?open\.spotify\.com/track/(\S{22})"#)?;

    if let Some(captures) = spotify_re.captures(url) {
        if let Some(m) = captures.get(1) {
            return Ok(SourceInfo {
                id: m.as_str(),
                service: Source::Spotify,
            });
        }
    }

    Err(Error::TrackError("Not valid input track URL".to_string()))
}
