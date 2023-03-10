[![CI](https://github.com/DarthB/bevy-jam-2-tba/actions/workflows/ci.yaml/badge.svg)](https://github.com/DarthB/bevy-jam-2-tba/actions/workflows/ci.yaml)

# Disastris - A bevy-jam-2 Game

Disastris - A disastrous factory meets tetris and you must fullfil the consumers' wishes! Therefore, disassemble the useless input blob that was delivered and combine the parts to something beautiful! Select tools and place them in the factory. Choose between several tool variants with the mouse wheel and hit the simulate button when you're ready.

A contribution to [bevy-jam-2](https://itch.io/jam/bevy-jam-2)

## How to Start

Use the following to run in debug/release mode:

- `cargo run`
- `cargo run --release`

You can also select a level via commandline arguments, at the moment only level 1-3 are supported:

- `cargo run -- -l3`

Find more command line options via:

- `cargo run -- --help`

Use the following to enable a egui based inspector

- `cargo run --features=debug`

## WASM

Does not work currently since the bevy engine update to 0.9.1

### Worked in previous revisions with bevy 0.8.\*

If not done yet install a wasm target and a server runner by:

- `rustup target add wasm32-unknown-unknown`
- `cargo install wasm-server-runner`

Than simply execute the runner a updated WASM build via:

- `cargo serve`

See [Cargo.toml](Cargo.toml) for the project setup and see [./cargo/config.toml](.cargo/config.toml) for for setup of aliaes.

When using a tag on github the WASM should be automaticially generated on github. For more information see the [Bevy GitHub CI Template](https://github.com/bevyengine/bevy_github_ci_template).

To generate the files that are needed for upload on a webserver install:

- `cargo install -f wasm-bindgen-cli`

and then use:

- `cargo build --release --target wasm32-unknown-unknown`
- `wasm-bindgen --out-dir ./wasm/ --target web ./target/wasm32-unknown-unknown/release/bevy_jam_2_disastris.wasm`
  This generates the files needed for an upload in wasm folder.
  See page on [Bevy Cheat Book](https://bevy-cheatbook.github.io/platforms/wasm.html) about WASM.

## Updates

Run the following commands for an update that ensures that WASM is still working

- `rustup update`
- `cargo update`
- `cargo install -f wasm-bindgen-cli`

# Based upon

[See this for CI/CD instructions](README_CI.md)
