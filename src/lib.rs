use rand::Rng;

pub mod blob;
pub mod bodies;
pub mod field;
pub mod game;
pub mod game_assets;
pub mod hud;
pub mod input;
pub mod level;
pub mod movement;
pub mod player_state;
pub mod render_old;
pub mod target;
pub mod turn;

pub const PX_PER_TILE: f32 = 32.0;
pub const SECONDS_PER_ROUND: f32 = 0.5;
pub const PX_PER_ICON: f32 = 64.0;

pub const QUOTE1: &str = "You won and all you get is this damn quote: \"Back in my days they delivered the raw materials in clean 4-block packages!\" by Gereon Bartel - Senior Block Composer";
pub const QUOTE2: &str = "You won and all you get is this damn quote: \"Rearranging ugly blobs into more elegant shapes is part of my daily routine!\" by Tim Janus - Fulltime Code Refactorer";
pub const QUOTE3: &str =
    "You won and all you get is this damn quote: \"I don't get it\" psi - Blockchain Expert";
pub const TUTORIAL: &str = "Disassemble the useless input blob that was delivered and combine the parts to something beautiful! Select tools and place them in the factory. Choose between several tool variants with the mouse wheel and hit the simulate button when you're ready.";

pub fn get_random_quote() -> String {
    let v = vec![QUOTE1, QUOTE2, QUOTE3];

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..v.len());
    v[idx].to_string()
}

pub const Z_FIELD: f32 = 0.0;
pub const Z_TRANS: f32 = 10.0;
pub const Z_SOLID: f32 = 20.0;
pub const Z_OVERLAY: f32 = 30.0;

pub mod prelude {
    pub use crate::blob::*;
    pub use crate::bodies::*;
    pub use crate::field::*;
    pub use crate::game::*;
    pub use crate::game_assets::*;
    pub use crate::hud::*;
    pub use crate::input::*;
    pub use crate::level::*;
    pub use crate::movement::*;
    pub use crate::player_state::*;
    pub use crate::render_old::*;
    pub use crate::target::*;
    pub use crate::turn::*;

    pub use crate::*;
}
