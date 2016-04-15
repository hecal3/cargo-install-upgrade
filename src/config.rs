use std::path::PathBuf;

/// Holds the settings of the current run
#[derive(Debug)]
pub struct Config {
    pub upgrade: bool,
    pub force: bool,
    pub verbose: bool,
    pub mode: PackageMode,
    pub cpath: PathBuf,
}

/// Settings for the current run
#[derive(Debug)]
pub enum PackageMode {
    All,
    Include(Vec<String>),
    Exclude(Vec<String>),
}
