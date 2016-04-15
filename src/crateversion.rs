use semver::Version;
use std::fmt;
use std::path::PathBuf;
use std::path::Path;
use self::PackageSource::*;
use util::*;
use tempdir::TempDir;
use config::Config;
use error::RemoteError;
use std::fs::{copy,DirBuilder};

#[derive(PartialEq, Debug, Clone)]
pub enum PackageSource {
    CratesIo,
    Git {
        url: String,
        commit: String,
        remote_commit: String,
    },
    Local {
        path: PathBuf,
    },
}

/// Version Information for Crates
#[derive(Clone, Debug)]
pub struct CrateVersion {
    pub name: String,
    pub version: Version,
    pub remote_version: Version,
    pub source: PackageSource,
    pub binaries: Vec<PathBuf>,
}

impl CrateVersion {
    pub fn new<S>(name: S, ver: Version) -> CrateVersion where S: Into<String> {
        CrateVersion {
            name: name.into(),
            version: ver.clone(),
            remote_version: ver,
            source: CratesIo,
            binaries: Vec::new(),
        }
    }

    pub fn new_fromstr<S>(name: S, ver: S) -> CrateVersion where S: AsRef<str> {
        CrateVersion::new(name.as_ref(), Version::parse(ver.as_ref()).unwrap())
    }

    /// Sets a remote git repository as package source
    pub fn set_repo<S>(&mut self, url: S, co: S) where S: Into<String> + Clone {
        self.source = Git {
            url: url.into(),
            commit: co.clone().into(),
            remote_commit: co.into(),
        };
    }

    /// Sets a local git repository as package source
    pub fn set_path<P>(&mut self, path: P) where P: Into<PathBuf> {
        self.source = Local { path: path.into() };
    }

    /// Sets binaries
    pub fn set_binaries(&mut self, bin: Vec<PathBuf>) {
        self.binaries.extend(bin);
    }

    /// Returns true if the package source is Crates.io
    pub fn is_cratesio(&self) -> bool {
        match self.source {
            CratesIo => true,
            _ => false,
        }
    }

    /// True if new remote version is available.
    pub fn new_remote_version(&self) -> bool {
        &self.version < &self.remote_version
    }

    /// Search the remote for new versions
    pub fn get_remote_version(&mut self, cfg: &Config) {
        let ver = match self.source {
            CratesIo => {
                match parse_cratesio(&self.name) {
                    Ok(out) => out,
                    Err(e) => {
                        println!("{} {}", e, self);
                        format!{"{}", self.version}
                    }
                }
            }
            Git{ref url, ref mut remote_commit, ..} => {
                match TempDir::new("tmprepo") {
                    Ok(tmpd) => {
                        let reppath = tmpd.path().to_path_buf();
                        cmd_run(&["git", "clone", "--depth=1", &url, reppath.to_str().unwrap()], cfg.verbose);
                        let out = match parse_cargo_toml(&reppath, "version") {
                            Ok(out) => out,
                            Err(e) => {
                                println!("{}", e);
                                format!{"{}", self.version}
                            }
                        };
                        let ncommit = cmd_return(&["git", "ls-remote", reppath.to_str().unwrap(), "HEAD"]);
                        remote_commit.clear();
                        remote_commit.push_str(&ncommit[..41]);
                        out
                    }
                    Err(e) => {
                        println!("Could not crate tempdir {}", e);
                        format!{"{}", self.version}
                    }
                }
            }
            Local{ ref path } => {
                match parse_cargo_toml(path, "version") {
                    Ok(out) => out,
                    Err(e) => {
                        println!("{}. Unable to extract version information", e);
                        format!{"{}", self.version}
                    }
                }
            }
        };
        debug!("Remote version, {}", &ver);
        self.remote_version = Version::parse(&ver).unwrap();
    }

    /// Upgrade package
    pub fn upgrade(&self, cfg: &Config) {
        println!("Update {}", self);
        if cfg.upgrade {
            if let Ok(ba) = self.backup(cfg) {
                self.uninstall();
                let success = self.install();
                if !success {
                    println!("Update not successful. Use backup");
                    self.reverse_backup(ba, cfg);
                }
            }
        }
    }

    fn uninstall(&self) {
        info!("Uninstall {}", self.name);
        cmd_run(&["cargo", "uninstall", &self.name], true);
    }

    fn install(&self) -> bool {
        info!("Install {}", self.name);
        let args = match self.source {
            CratesIo => vec!["cargo", "install", &self.name],
            Git{ref url, ..} => vec!["cargo", "install", "--git", &url],
            Local{ref path} => vec!["cargo", "install", "--path", path.to_str().unwrap()],
        };
        cmd_run(&args, true)
    }

    fn backup(&self, cfg: &Config) -> Result<TempDir,&str> {
        if let Ok(tmpd) = TempDir::new(&self.name) {
            let mut reppath = tmpd.path().to_path_buf();
            reppath.push("bin");
            DirBuilder::new()
                .recursive(true)
                .create(&reppath).unwrap();
            for binary in &self.binaries {
                reppath.push(binary.file_name().unwrap().to_str().unwrap());
                let _ = copy(binary, &reppath);
                reppath.pop();
            }
            reppath.pop();
            let mut datpath = cfg.cpath.clone();
            datpath.push(".crates.toml");
            reppath.push(".crates.toml");
            let _ = copy(&datpath, &reppath);
        return Ok(tmpd)
        } else {
            return Err("could not create tempdir")
        };
    }

    fn reverse_backup(&self, dir: TempDir, cfg: &Config) {
        let mut tmppath = dir.path().to_path_buf();
        let mut cargopath = cfg.cpath.clone();
        tmppath.push("bin");
        cargopath.push("bin");
        for binary in &self.binaries {
            let filename = binary.file_name().unwrap().to_str().unwrap();
            tmppath.push(filename);
            cargopath.push(filename);
            let _ = copy(&tmppath, &cargopath);
            tmppath.pop();
            cargopath.pop();
        }
        tmppath.pop();
        cargopath.pop();
        tmppath.push(".crates.toml");
        cargopath.push(".crates.toml");
        let _ = copy(&tmppath, &cargopath);
    }
}

impl fmt::Display for CrateVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.source {
            CratesIo => {
                write!(f,
                       "{} ({}) -> ({})",
                       self.name,
                       self.version,
                       self.remote_version)
            }
            Git{ref url, ref commit, ref remote_commit} => {
                write!(f,
                       "{} ({}):{} -> ({}):{} {}",
                       self.name,
                       self.version,
                       commit,
                       self.remote_version,
                       remote_commit,
                       url)
            }
            Local{ path: ref p} => {
                write!(f,
                       "{} ({}) -> ({}) {}",
                       self.name,
                       self.version,
                       self.remote_version,
                       p.display())
            }
        }
    }
}

fn parse_cratesio(cratename: &str) -> Result<String, RemoteError> {
    let input = cmd_return(&["cargo", "search", cratename]);
    let line = match input.lines()
            .filter(|x| x.starts_with(&format!("{} ", cratename))).nth(0) {
        Some(line) => line,
        None => return Err(RemoteError::NoCrate(cratename.to_owned())),
    };
    match line.split(|c| c == '(' || c == ')').map(|s| s.trim()).nth(1) {
        Some(val) => Ok(val.to_owned()),
        None => Err(RemoteError::ParseError(String::from("Could not parse cargo search output"))),
    }
}

fn parse_cargo_toml<P,S>(path: P, field: S) -> Result<String, RemoteError>
                where P: AsRef<Path>, S: AsRef<str> {
    use serde_json::{self, Value};
    let pa = path.as_ref().join("Cargo.toml");
    if !pa.is_file() {
        return Err(RemoteError::NoCargoToml(format!("Error: {} is no file.", pa.to_str().unwrap().to_owned())));
    }

    let pa = match pa.to_str() {
        Some(pa) => pa,
        None => return Err(RemoteError::NoCargoToml("could not read Cargo.toml".to_owned())),
    };

    //let pa = format!("{}/Cargo.toml", path);
    let input = cmd_return(&["cargo", "read-manifest", "--manifest-path", pa]);
    trace!("{}", &input);

    let val: Value = serde_json::from_str(&input).unwrap_or_else(|e| panic!("open {}", e));
    let obj = val.as_object().unwrap();
    trace!("{:?}", obj);

    let verstr = obj.get(field.as_ref()).unwrap();
    if let Value::String(ref v) = *verstr {
        debug!("Version: {}", v);
        return Ok(v.to_owned());
    }
    Err(RemoteError::CargoToml(format!("Error parsing {}", pa)))
}
