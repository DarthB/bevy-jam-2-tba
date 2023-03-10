//! This crate defines the Disastris binaries. Disastris is a contribution to bevy-jam-2.
//!
//! Here the lib [`::clap`] is used to implement a command line, which acts as possibility to configure the game instance,
//! atm only the level can be given.
//!
//! ## TODOs
//! - [x] Choose level
//! - [ ] Use folders
//! - [ ] Startup in different game states, e.g. mainmenu vs. ingame, etc.
//!
//! Most of its functionality and therefore the documentation relies in the [`::bevy_jam_2_disastris_lib`].

use std::{env, path::PathBuf};

use bevy_jam_2_disastris_lib::start_disastris;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Clone, Copy, Default)]
pub enum CliCommands {
    #[default]
    /// Starts the game, DEFAULT
    Game,

    /// Runs the level creation code and stores the files as RON
    DumpLevelsFromCode,
}

#[derive(Parser)]
#[command(author="Gereon Bartel, Tim Janus and Philipp Sieweck", version="0.2.0", about="A game for the bevy-jam2", long_about=None)]
struct Cli {
    /// the number of the level that should be loaded at game startup
    #[arg(short, long, value_name = "LVL_NO", default_value_t = 1)]
    level: u32,

    #[arg(short, long, value_name = "RES_FOLDER")]
    /// path to the folder containing the resources / assets of the game
    resource_folder: Option<PathBuf>,

    #[arg(short, long, value_name = "OUTPUT_FOLDER")]
    /// path that shall be used to generate outputs, e.g. screenshots
    output_folder: Option<PathBuf>,

    #[command(subcommand)]
    /// the applied command
    subcommand: Option<CliCommands>,
}

impl Cli {
    /// This function builds parameters out of the input provided via the command line interface.
    ///
    /// In short is replaces all options with actual (default) values and makes folders absolute if
    /// they are given relative.
    pub fn build_parameters(self) -> CliParameters {
        // read cli or generate default values
        let subcommand = self.subcommand.unwrap_or(CliCommands::Game);
        let rel_res_folder = self.resource_folder.unwrap_or("assets/".into());
        let rel_out_folder = self.output_folder.unwrap_or("target".into());

        // build absolute folders based on current working directory
        let cwd = env::current_dir().expect("No current working directory - May crash WASM?");

        // ensure resource folder is absolute
        let abs_res_folder = if rel_res_folder.is_absolute() {
            rel_res_folder
        } else {
            let mut abs_res_folder = cwd.clone();
            abs_res_folder.push(rel_res_folder);
            abs_res_folder
        };

        // ensure output folder is absolute
        let abs_out_folder = if rel_out_folder.is_absolute() {
            rel_out_folder
        } else {
            let mut abs_out_folder = cwd;
            abs_out_folder.push(rel_out_folder);
            abs_out_folder
        };

        CliParameters {
            level_num: self.level,
            resource_folder: abs_res_folder,
            output_folder: abs_out_folder,
            subcommand,
        }
    }
}

struct CliParameters {
    level_num: u32,

    resource_folder: PathBuf,

    output_folder: PathBuf,

    subcommand: CliCommands,
}

/// Setups a bevy app object and adds the default plugins, systems and events
fn main() {
    let cli = Cli::parse();
    let cli = cli.build_parameters();

    match cli.subcommand {
        CliCommands::Game => start_disastris(cli.resource_folder, cli.level_num),
        CliCommands::DumpLevelsFromCode => {
            println!(
                "TODO: write output to: {}",
                cli.output_folder.to_str().unwrap_or("INVALID PATH")
            );
        }
    }
}
