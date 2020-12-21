extern crate dirs;

use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::*;
use std::fs::File;
use std::io::prelude::Read;

use crate::error::UpgradeError;
use crate::config::Config;
use crate::crateversion::{CrateVersion,Result};

pub fn cmd_run(cmd: &[&str], verbose: bool) -> bool {
    info!("run command: {}", cmd.join(" "));
    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);
    if !verbose {
        command.stdout(Stdio::null())
               .stderr(Stdio::null())
               .stdin(Stdio::null());
    }
    let ret = command.status();
    match ret {
        Ok(exit) => exit.success(),
        Err(_) => {
            println!("Error running: {}", &cmd.join(" "));
            false
        },
    }
}

pub fn cmd_return(cmd: &[&str]) -> String {
    info!("ret command: {}", cmd.join(" "));
    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);
    let out = command.output().unwrap_or_else(|e| panic!("{}", e));
    String::from_utf8(out.stdout).unwrap_or_else(|_| "ERROR".to_owned())
}


pub fn search_cargo_data() -> Option<PathBuf> {
    debug!("Search for cargohome");
    let mut candidates = Vec::new();
    if cfg!(target_os = "windows") {
        candidates.push("AppData/Local/.multirust/cargo");
    }   
    candidates.push(".cargo");
    let mut retpath: Option<PathBuf> = None;

    for p in candidates {
        let mut path: PathBuf = dirs::home_dir()?;
        path.push(p);
        retpath = Some(path.clone());
        debug!("dir: {}", path.display());
        path.push(".crates.toml");
        debug!("file: {}", path.display());
        if metadata(path).is_ok() {
            break;
        } else {
            retpath = None;
        };
    }
    retpath
}


pub fn read_installed_packages(cfg: &Config) -> Result<Vec<CrateVersion>> {
    let mut path = cfg.cpath.clone();
    path.push(".crates.toml");
    let mut out = Vec::new();

    let mut file = File::open(&path)?;
    let mut s = String::new();
    let _ = file.read_to_string(&mut s);
    let toml = s.parse::<toml::Value>()?;

    if !toml.is_table() {
        return Ok(out);
    }

    let vals = toml.as_table()
        .ok_or_else(|| UpgradeError::from("Toml not valid"))?;

    for v in vals.values() {
        if !v.is_table() {
            continue;
        }

        let stable = v.as_table().ok_or_else(|| UpgradeError::from("No Table"))?;
        for (k2, v2) in stable {
            let crat = k2.as_str().to_owned();
            let elements: Vec<&str> = crat.split(' ').collect();
            let address = elements[2].trim_matches(|c| c == '(' || c == ')');
            let mut topush = CrateVersion::new_fromstr(elements[0], elements[1]);
            let addr: Vec<&str> = address.split('+').collect();
            match addr[0] {
                "git" => {
                    let mut elem = addr[1].split('#');
                    topush.set_repo(elem.next().unwrap(), elem.next().unwrap());
                },
                "path" if cfg!(target_os = "windows") => {
                    topush.set_path(addr[1].trim_start_matches("file:///"));
                },
                "path" => {
                    topush.set_path(addr[1].trim_start_matches("file://"));
                },
                _ => {},
            };

            if let Some(binaries) = v2.as_array() {
                let bin: Vec<&str> = binaries.iter().map(|x| x.as_str().unwrap()).collect();
                
                let mut paths_binaries = Vec::new();
                for binaryname in bin {
                    let mut path = cfg.cpath.clone();
                    path.push("bin");
                    path.push(binaryname);
                    paths_binaries.push(path);
                }
                topush.set_binaries(&paths_binaries)
            }
            debug!("{:?}", topush);
            out.push(topush);
        }
    }
    Ok(out)
}
