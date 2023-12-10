[![CI](https://github.com/DarthB/bevy-jam-2-tba/actions/workflows/ci.yaml/badge.svg)](https://github.com/DarthB/bevy-jam-2-tba/actions/workflows/ci.yaml)

# Disastris

Disastris - A disastrous factory meets tetris and you must fullfil the consumers' wishes! Therefore, disassemble the useless input blob that was delivered and combine the parts to something beautiful! Select tools and place them in the factory. Choose between several tool variants with the mouse wheel and hit the simulate button when you're ready. An somehow older version can be found on [itch.io](https://tjanus.itch.io/disastris)

It started as a contribution to [bevy-jam-2](https://itch.io/jam/bevy-jam-2)

It has been continued on [Dragons Christmas Jam 2023](https://itch.io/jam/rustcpp-dragons-christmas-jam-2023)

## How to Start

Use the following to run in debug/release mode:

- `cargo run`
- `cargo run --release`

You can select a starting application state via commandline arguments:

- - `cargo run -- -sAnimationTest` - 'Mainmenu', 'PlayLevel' and 'AnimationTest'.

You can also select a level for the 'PlayLevel' state via commandline arguments, at the moment only level 1-3 are supported:

- `cargo run -- -l3`

Find more command line options via:

- `cargo run -- --help`

Use the following to enable a egui based inspector

- `cargo run --features=debug`

## WASM



# Based upon

[See this for CI/CD instructions](README_CI.md)
