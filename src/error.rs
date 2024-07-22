use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug)]
struct MatchNotFoundError;

impl Error for MatchNotFoundError {}

impl Display for MatchNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
