# cargo-install-upgrade

Updates all binaries installed by `cargo install` to the latest versions.
The feature is left out of Cargo by design. This is just a small convenience wrapper.

The old version will be kept as backup for the duration of the build. In case of build failure, it will roll back.

Supports the following sources:
- Crates.io (`cargo install`)
- local repositorys (`cargo install --path`)
- git repositorys (`cargo install --git`)

This command will keep itself up to date. (On Unix systems)

## Installation
```
cargo install --git https://github.com/hecal3/cargo-install-upgrade
```

## Usage
```
cargo install-upgrade
```
See `cargo install-upgrade -h` for more information.
