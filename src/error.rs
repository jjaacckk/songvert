// sources:
// - https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html
// - https://fettblog.eu/rust-enums-wrapping-errors/

use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DatabaseError(String),
    TrackError(String),
    DownloadError(String),
    TagError(String),
    RegexError(regex::Error),
    JsonError(serde_json::Error),
    ParseIntError(std::num::ParseIntError),
    RetrievalError(reqwest::Error),
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DatabaseError(s) => write!(f, "Database Error: {}", s),
            Error::TrackError(s) => write!(f, "Track Error: {}", s),
            Error::DownloadError(s) => write!(f, "Download Error: {}", s),
            Error::TagError(s) => write!(f, "Tag Error: {}", s),
            Error::JsonError(e) => write!(f, "JSON Parsing Error: {}", e),
            Error::RegexError(e) => write!(f, "Regex Error: {}", e),
            Error::ParseIntError(e) => write!(f, "Integer Parsing Error: {}", e),
            Error::RetrievalError(e) => write!(f, "Rerieval Error: {}", e),
            Error::IoError(e) => write!(f, "IO Error: {}", e),
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
        Error::JsonError(err)
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

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}
// impl From<id3::Error> for Error {
//     fn from(err: id3::Error) -> Self {
//         Error::TagError(err)
//     }
// }
