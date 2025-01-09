extern crate dirs;

use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::*;
use std::fs::File;
use std::io::prelude::Read;

use crate::error::UpgradeError;
use crate::config::Config;
use crate::crateversion::{CrateVersion,Result};
use serde_json::Value;

pub fn cmd_run(cmd: &[&str], verbose: bool) -> bool {
    info!("run command: {:?}", cmd);
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
    path.push(".crates2.json");
    let mut out = Vec::new();

    let mut file = File::open(&path)?;
    let mut s = String::new();
    let _ = file.read_to_string(&mut s);

    let v: Value = serde_json::from_str(&s)?;

    let installs = v["installs"].as_object().ok_or_else(|| UpgradeError::from("json not valid"))?;
    for (key, value) in installs {

        let crat = key.as_str().to_owned();
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

        let details = value.as_object().ok_or_else(|| UpgradeError::from("details-segment not valid"))?;
        
        if let Some(binaries) = details["bins"].as_array() {
            let mut paths_binaries = Vec::new();
            for binaryvalue in binaries {
                if let Some(binarystr) = binaryvalue.as_str() {
                    let mut path = cfg.cpath.clone();
                    path.push("bin");
                    path.push(binarystr);
                    paths_binaries.push(path);
                }
            }
            topush.set_binaries(&paths_binaries);
        }



        if let Some(features) = details["features"].as_array() {
            let mut feature_list = Vec::new();
            for binaryvalue in features {
                if let Some(binarystr) = binaryvalue.as_str() {
                    feature_list.push(String::from(binarystr));
                }
                topush.set_features(&feature_list);
            }
        }

        debug!("{:?}", topush);
        out.push(topush);
    }
    Ok(out)
}
