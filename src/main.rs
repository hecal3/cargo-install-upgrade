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

use clap::{App, Arg, AppSettings, SubCommand};

use std::path::PathBuf;

use crate::crateversion::CrateVersion;
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
