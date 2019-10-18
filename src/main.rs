//! Cargo Subcommand to update packages installed via cargo install
//!
//! # Usage
//!
//! `cargo install-upgrade`

#[macro_use] extern crate clap;
extern crate semver;
extern crate serde_json;
extern crate tempdir;
extern crate toml;

#[macro_use] extern crate log;
extern crate env_logger;

mod util;
mod crateversion;
mod config;
mod error;

use clap::{App, Arg, AppSettings, SubCommand};

use std::fs::File;
use std::io::prelude::Read;
use std::path::PathBuf;

use crate::crateversion::{CrateVersion,Result};
use crate::config::*;
use crate::util::*;
use crate::error::UpgradeError;

fn main() {
    env_logger::init();

    let m = App::new("cargo-install-upgrade")
        .author("hecal3 <hecal3@users.noreply.github.com>")
        .about("Updates crates installed with cargo install")
        .version(&*format!("v{}", crate_version!()))
        .bin_name("cargo")
        .settings(&[AppSettings::GlobalVersion,
                    AppSettings::SubcommandRequired])
        .subcommand(SubCommand::with_name("install-upgrade")
            .about("Updates crates installed with cargo install")
            .args_from_usage(
                "-p, --packages [PKG]...   'Crates to upgrade (defaults to all)'
                -f, --force                'Force a reinstall of git/local packages'
                -v, --verbose              'Verbose output'
                -c, --cargo [DIR]          'Path to Cargo home directory'
                -d, --dry-run              'Do not perform actual upgrades'")
            .arg(Arg::from_usage(
                "-e, --exclude [PKG]...    'Crates to exclude'")
                .conflicts_with("packages")
                    ))
        .get_matches();

    if let Some(m) = m.subcommand_matches("install-upgrade") {
        let mode = match (m.values_of_lossy("packages"), m.values_of_lossy("exclude")) {
            (None, None) => PackageMode::All,
            (Some(m), None) => PackageMode::Include(m),
            (None, Some(m)) => PackageMode::Exclude(m),
            (Some(_), Some(_)) => unreachable!(),
        };

        let home = match m.value_of("cargo") {
            Some(val) => Some(PathBuf::from(val)),
            None => search_cargo_data(),
        };

        if let Some(home) = home {
            let cfg = Config {
                upgrade: !m.is_present("dry-run"),
                force: m.is_present("force"),
                verbose: m.is_present("verbose"),
                mode,
                cpath: home,
            };
            debug!("{:?}", cfg);
            execute(cfg);
        } else {
            println!("Could not find cargo home directory. Please set it manually with -c.");
        }
    }
}

fn execute(cfg: Config) {
    info!("Searc for packages");
    let mut installed: Vec<CrateVersion> = read_installed_packages(&cfg).unwrap();
    info!("Found packages: {:?}", installed);

    match cfg.mode {
        PackageMode::Include(ref pack) => {
            installed.retain(|x| pack.contains(&x.name));
        },
        PackageMode::Exclude(ref pack) => {
            installed.retain(|x| !pack.contains(&x.name));
        },
        _ => {},
    };

    for cr in &mut installed {
        debug!("before: {}", cr);
        cr.get_remote_version(&cfg);
        debug!("after: {}", cr);

        match (cr.new_remote_version(), cfg.force, cr.is_cratesio()) {
            (true,_,_) | (_,true,false) => cr.upgrade(&cfg),
            (false,false,false) =>
                println!("{} is a local/git package. Force an upgrade with -f", cr),
            _ => println!("{} is up to date.", cr),
        }
    }

    if let PackageMode::Include(ref packages) = cfg.mode {
        let installed: Vec<String> = installed.into_iter().map(|x| x.name).collect();
        for n in packages.iter().filter(|x| !installed.contains(x)) {
            println!("{} is not installed.", n);
        }
    }
}

fn read_installed_packages(cfg: &Config) -> Result<Vec<CrateVersion>> {
    let mut path = cfg.cpath.clone();
    path.push(".crates.toml");
    let mut out = Vec::new();

    let mut file = File::open(&path)?;
    let mut s = String::new();
    let _ = file.read_to_string(&mut s);
    let mut parser = toml::Parser::new(&s);
    let toml = match parser.parse() {
        Some(toml) => toml,
        None => return Err(UpgradeError::Parse(String::from("Could not read Toml"))),
    };

    for v in toml.values() {
        if let Some(stable) = v.as_table() {
            for (k2,v2) in stable {
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

                if let Some(binaries) = v2.as_slice() {
                    let bin: Vec<&str> = binaries.iter().map(|x| x.as_str().unwrap()).collect();
                    let mut binar = Vec::new();
                    for stri in bin {
                        let mut path = cfg.cpath.clone();
                        path.push("bin");
                        path.push(stri);
                        binar.push(path);
                    }
                    //println!("{:?}", binar);
                    topush.set_binaries(&binar)
                }
                debug!("{:?}", topush);
                out.push(topush);
            }
        }
    }
    Ok(out)
}
