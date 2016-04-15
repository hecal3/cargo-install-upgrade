use std::fmt::{Display, Formatter, Result};
use std::error::Error;
use self::RemoteError::*;

#[derive(Debug)]
pub enum RemoteError {
    NoCargoToml(String),
    CargoToml(String),
    NoCrate(String),
    ParseError(String),
}

impl Display for RemoteError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.description())
    }
}

impl Error for RemoteError {
    fn description(&self) -> &str {
        match *self {
            ParseError(..) => "parse error",
            NoCargoToml(ref s) | CargoToml(ref s) | NoCrate(ref s) => &*s,
        }
    }   
}
