use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct MatchError;

impl Error for MatchError {}

impl Display for MatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error matching track(s)")
    }
}

#[derive(Debug)]
pub struct CreateError;

impl Error for CreateError {}

impl Display for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error creating track(s)")
    }
}

#[derive(Debug)]
pub struct SpotifyError;

impl Error for SpotifyError {}

impl Display for SpotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error accessing Spotify")
    }
}
