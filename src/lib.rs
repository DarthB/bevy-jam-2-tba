pub mod blob;
pub mod bodies;
pub mod field;
pub mod game;
pub mod game_assets;
pub mod input;
pub mod movement;
pub mod turn;

pub const PX_PER_TILE: f32 = 32.0;
pub const SECONDS_PER_ROUND: f32 = 0.5;

pub const Z_FIELD: f32 = 0.0;
pub const Z_TRANS: f32 = 10.0;
pub const Z_SOLID: f32 = 20.0;
pub const Z_OVERLAY: f32 = 30.0;

pub mod prelude {
    pub use crate::blob::*;
    pub use crate::bodies::*;
    pub use crate::field::*;
    pub use crate::game::*;
    pub use crate::game_assets;
    pub use crate::input::*;
    pub use crate::movement::*;
    pub use crate::turn::*;

    pub use crate::*;
}
