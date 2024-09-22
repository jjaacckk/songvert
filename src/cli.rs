use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Easily convert URLs between Spotify, Apple Music, Bandcamp, and YouTube
#[derive(Parser)]
#[command(name= "Songvert",version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: CommandType,
}

#[derive(Subcommand)]

enum CommandType {
    Playlist {
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        output: Output,
    },
    Track {
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        output: Output,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Input {
    /// Create from URL
    #[arg(short, long)]
    url: Option<String>,

    /// Load from JSON file
    #[arg(short, long, value_name = "FILE")]
    input_file: Option<PathBuf>,
}

#[derive(Args)]
#[group(multiple = true)]
struct Output {
    /// Download from best source
    #[arg(short, long)]
    download: bool,

    /// Save all JSON metadata to file
    #[arg(short, long, value_name = "FILE")]
    output_file: Option<PathBuf>,
}

#[derive(Args)]
#[group(multiple = true)]
struct ConversionServices {
    #[arg(short)]
    spotify: bool,
    #[arg(short)]
    apple_music: bool,
    #[arg(short)]
    bandcamp: bool,
    #[arg(short)]
    youtube: bool,
}
