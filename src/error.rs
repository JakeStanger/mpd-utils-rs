use mpd_client::client::CommandError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    NoHostConnectedError,
    CommandError(CommandError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::NoHostConnectedError => "No host connected".to_string(),
                Error::CommandError(err) => err.to_string(),
            }
        )
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
