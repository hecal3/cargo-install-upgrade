use std::fmt::{Display, Formatter, Result};
use std::error::Error;
use std::io;
use self::UpgradeError::*;

#[derive(Debug)]
pub enum UpgradeError {
    Parse(String),
    NoCrate(String),
    Gen(String),
    Io(io::Error),
    SerdeError(serde_json::Error),
}

impl Display for UpgradeError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Parse(ref s) => write!(f, "Parse Error {}", &s),
            NoCrate(ref s) => write!(f, "{}", &s),
            Gen(ref s) => write!(f, "{}", &s),
            Io(ref err) => err.fmt(f),
            SerdeError(ref err) => err.fmt(f),
        }
    }
}

impl Error for UpgradeError {

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Io(ref err) => Some(err),
            SerdeError(ref err) => Some(err),
            _ => None
        }
    }
}

impl From<&str> for UpgradeError {
    fn from(s: &str) -> UpgradeError {
        UpgradeError::Gen(s.to_owned())
    }
}

impl From<io::Error> for UpgradeError {
    fn from(err: io::Error) -> UpgradeError {
        UpgradeError::Io(err)
    }
}

impl From<serde_json::Error> for UpgradeError {
    fn from(err: serde_json::Error) -> UpgradeError {
        UpgradeError::SerdeError(err)
    }
}
