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

use clap::{Parser, Subcommand};
use disastris_lib::{start_disastris, GameConfig};

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

    /// The state in that Disastris shall shart, supported are 'Mainmenu', 'PlayLevel' and 'AnimationTest'.
    #[arg(short, long, value_name = "START_STATE", default_value = "PlayLevel")]
    start_state: String,

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
        let rel_out_folder = self.output_folder.unwrap_or("target".into());

        // build absolute folders based on current working directory
        let cwd = env::current_dir();
        let abs_out_folder = match cwd {
            Ok(cwd) => {
                // ensure output folder is absolute
                if rel_out_folder.is_absolute() {
                    rel_out_folder
                } else {
                    let mut abs_out_folder = cwd;
                    abs_out_folder.push(rel_out_folder);
                    abs_out_folder
                }
            }
            Err(_) => PathBuf::from("."),
        };

        CliParameters {
            level_num: self.level,
            output_folder: abs_out_folder,
            start_state: self.start_state,
            subcommand,
        }
    }
}

struct CliParameters {
    level_num: u32,

    start_state: String,

    output_folder: PathBuf,

    subcommand: CliCommands,
}

/// Setups a bevy app object and adds the default plugins, systems and events
fn main() {
    let cli = Cli::parse();
    let cli = cli.build_parameters();

    match cli.subcommand {
        CliCommands::Game => {
            let config = GameConfig {
                start_level: cli.level_num,
                start_state: cli.start_state,
                state_from_placeholder: disastris_lib::DisastrisAppState::PlayLevel,
            };
            start_disastris(config);
        }
        CliCommands::DumpLevelsFromCode => {
            println!(
                "TODO: write output to: {}",
                cli.output_folder.to_str().unwrap_or("INVALID PATH")
            );
        }
    }
}
