//! Cargo Subcommand to update packages installed via cargo install
//!
//! # Usage
//!
//! `cargo install-upgrade`

#[macro_use] extern crate clap;
extern crate semver;
extern crate serde_json;
extern crate tempdir;

#[macro_use] extern crate log;
extern crate env_logger;

mod util;
mod crateversion;
mod config;
mod error;

use clap::Command;

use std::path::PathBuf;

use crate::crateversion::CrateVersion;
use crate::config::*;
use crate::util::*;
use crate::error::UpgradeError;

fn main() {
    env_logger::init();
    

    let m = Command::new("cargo-install-upgrade")
        .author("hecal3")
        .about("Updates crates installed with cargo install")
        .version("1.0.15")
        .bin_name("cargo")
        .propagate_version(true)
        .subcommand_required(true)
        .subcommand(Command::new("install-upgrade")
            .about("Updates crates installed with cargo install")
            .args(&[
               arg!(-p --packages [PKG]...   "Crates to upgrade (defaults to all)"),
               arg!(-f --force               "Force a reinstall of git/local packages"),
               arg!(-v --verbose             "Verbose output"),
               arg!(-c --cargo [DIR]         "Path to Cargo home directory"),
               arg!(-d --dry-run             "Do not perform actual upgrades'"),
               arg!(-e -exclude [PKG]        "crates to exclude").conflicts_with("package")
            ])
        ).get_matches();


    if let Some(m) = m.subcommand_matches("install-upgrade") {

        let mode = match (m.get_many::<String>("packages"), m.get_many::<String>("exclude")) {
            (None, None) => PackageMode::All,
            (Some(m), None) => PackageMode::Include(m.map(|s| String::from(s)).collect()),
            (None, Some(m)) => PackageMode::Exclude(m.map(|s| String::from(s)).collect()),
            (Some(_), Some(_)) => unreachable!(),
        };

        let home = match m.get_one::<String>("cargo").map(|s| s.as_str()) {
            Some(val) => Some(PathBuf::from(val)),
            None => search_cargo_data(),
        };

        if let Some(home) = home {
            let cfg = Config {
                upgrade: !m.get_one::<bool>("dry-run").map_or_else(|| false, |b| *b),
                force: m.get_one::<bool>("force").map_or_else(|| false, |b| *b),
                verbose: m.get_one::<bool>("verbose").map_or_else(|| false, |b| *b),
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

    for crate_version in &mut installed {
        debug!("before: {}", crate_version);
        crate_version.get_remote_version(&cfg);
        debug!("after: {}", crate_version);

        match (crate_version.new_remote_version(), cfg.force, crate_version.is_cratesio()) {
            (true,_,_) | (_,true,false) => crate_version.upgrade(&cfg),
            (false,false,false) =>
                println!("{} is a local/git package. Force an upgrade with -f", crate_version),
            _ => println!("{} is up to date.", crate_version),
        }
    }

    if let PackageMode::Include(ref packages) = cfg.mode {
        let installed: Vec<String> = installed.into_iter().map(|x| x.name).collect();
        for n in packages.iter().filter(|x| !installed.contains(x)) {
            println!("{} is not installed.", n);
        }
    }
}
