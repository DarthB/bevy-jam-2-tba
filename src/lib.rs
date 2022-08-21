pub mod blob;
pub mod input;
pub mod game;
pub mod turn;
pub mod movement;

pub const PX_PER_TILE: f32 = 32.0;
pub const SECONDS_PER_ROUND: f32 = 0.75;

pub mod prelude {
    pub use crate::input::*;
    pub use crate::game::*;
    pub use crate::movement::*;
    pub use crate::turn::*;
}