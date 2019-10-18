use std::fmt::{Display, Formatter, Result};
use std::error::Error;
use std::io;
use self::UpgradeError::*;

#[derive(Debug)]
pub enum UpgradeError {
    Parse(String),
    NoCrate(String),
    Io(io::Error),
}

impl Display for UpgradeError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.description())
    }
}

impl Error for UpgradeError {
    fn description(&self) -> &str {
        match *self {
            Parse(..) => "Parse Error",
            NoCrate(ref s) => &*s,
            Io(ref err) => err.description(),
        }
    }   

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            Io(ref err) => Some(err as &dyn Error),
            _ => None,
        }
    }
}

impl From<io::Error> for UpgradeError {
    fn from(err: io::Error) -> UpgradeError {
        UpgradeError::Io(err)
    }
}
