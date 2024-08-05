// sources:
// - https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html
// - https://fettblog.eu/rust-enums-wrapping-errors/

use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MatchError,
    FindError,
    CreateError, //KeyError??
    SessionGrabError,
    RegexError(regex::Error),
    ParseError(serde_json::Error),
    RetrievalError(reqwest::Error),
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MatchError => write!(f, "unable to find match"),
            Error::FindError => write!(f, "unable to find track(s) from given parameter(s)"),
            Error::CreateError => write!(f, "unable to create track (probably key error)"),
            Error::SessionGrabError => write!(f, "unable to grab session info"),
            Error::RegexError(e) => write!(f, "{}", e),
            Error::ParseError(e) => write!(f, "{}", e),
            Error::RetrievalError(e) => write!(f, "{}", e),
            Error::IoError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    // fn source(&self) -> Option<&(dyn std::error:Error + 'static)> {
    //     match *self {
    //         Error::TrackMatchError => None,
    //         Error::TrackFindError => None,
    //         Error::SessionGrabError => None,
    //         Error::RegexError(ref e) => Some(e),
    //         Error::ParseError(ref e) => Some(e),
    //         Error::RetrievalError(ref e) => Some(e),
    //         Error::IoError(ref e) => Some(e)
    //     }
    // }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::ParseError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::RetrievalError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::RegexError(err)
    }
}
