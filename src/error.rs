use crate::tool::error::Error as ToolError;
use semver;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
};

#[derive(Debug)]
pub enum Error {
    HomeDir,
    InvalidVersion(semver::SemVerError),
    Io(IoError),
    Tool(ToolError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::HomeDir => write!(f, "failed to find your home directory!"),
            Self::InvalidVersion(v) => write!(f, "invalid version: {}", v),
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Tool(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::Io(e)
    }
}

impl From<semver::SemVerError> for Error {
    fn from(e: semver::SemVerError) -> Self {
        Self::InvalidVersion(e)
    }
}

impl From<ToolError> for Error {
    fn from(e: ToolError) -> Self {
        Self::Tool(e)
    }
}
