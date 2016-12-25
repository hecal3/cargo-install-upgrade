use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::*;
use std::env::home_dir;

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
    String::from_utf8(out.stdout).unwrap_or("ERROR".to_owned())
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
        let mut path: PathBuf = home_dir().unwrap();
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
