pub mod input;
pub mod game;
pub mod movement;

pub mod prelude {
    pub use crate::input::*;
    pub use crate::game::*;
    pub use crate::movement::*;
}