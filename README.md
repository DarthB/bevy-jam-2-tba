# TITLE TBA - A bevy-jam-2 Game

A contribution to [bevy-jam-2](https://itch.io/jam/bevy-jam-2)

## How to Start

Use the following to run in debug/release mode:
* `cargo run`
* `cargo run --release`

The left bird is controlled with WASD and F for powerups.

The right bird is controlled with arrow keys and RShift for powerups.

Use the following to enable a egui based inspector
* `cargo run --features=debug`

## WASM

If not done yet install a wasm server runner by:
* `cargo install wasm-server-runner`

Than simply execute the runner a updated WASM build via:
* `cargo serve`

See [Cargo.toml](Cargo.toml) for the project setup and see [./cargo/config.toml](.cargo/config.toml) for for setup of aliaes.

When using a tag on github the WASM should be automaticially generated on github. For more information see the [Bevy GitHub CI Template](https://github.com/bevyengine/bevy_github_ci_template). 

To generate the files that are needed for upload on a webserver install:
* `cargo install -f wasm-bindgen-cli`

and then use:

* `cargo build --release --target wasm32-unknown-unknown`
* `wasm-bindgen --out-dir ./wasm/ --target web ./target/wasm32-unknown-unknown/release/bevy_jam_2_tba.wasm`
This generates the files needed for an upload in wasm folder.
See page on [Bevy Cheat Book](https://bevy-cheatbook.github.io/platforms/wasm.html) about WASM.

# Based upon 
[See this for CI/CD instructions](README_CI.md)
