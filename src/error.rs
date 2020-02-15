use semver;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};
use zip::result::ZipError;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::StatusCode),
    Reqwest(reqwest::Error),
    Io(io::Error),
    InvalidVersion(semver::SemVerError),
    Zip(ZipError),
    HomeDir,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Http(s) => write!(f, "http error: {}", s),
            Error::Reqwest(e) => write!(f, "http error: {}", e),
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::InvalidVersion(v) => write!(f, "invalid version: {}", v),
            Error::HomeDir => write!(f, "failed to find your home directory!"),
            Error::Zip(e) => write!(f, "zip: {}", e),
        }
    }
}

impl StdError for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<semver::SemVerError> for Error {
    fn from(e: semver::SemVerError) -> Self {
        Self::InvalidVersion(e)
    }
}

impl From<ZipError> for Error {
    fn from(e: ZipError) -> Self {
        Self::Zip(e)
    }
}
