//! The module contains data that form the game, e.g. assets, levels and auxillary structures.
//!
//! A [`level::Level`] defines a Disastris puzzle that shall be solved by the player via [super::field::tool::Tool]s
//!
//! The [`bodies`] module is quite important to design [`super::field::blob::Blob`]s and [`super::field::target::Target`]s.
//! For this a [`bodies::BodyDefinition`] structure is used that consists of a size, pivot and grid information.
//!
//! The strucuture [`assets::GameAssets`] contains references to every asset that is used throughout the game
//!

pub mod assets;
pub mod bodies;
pub mod level;

pub mod prelude {
    pub use super::bodies::gen_random_tetris_body;
    pub use super::bodies::gen_tetris_body;
    pub use super::bodies::BodyDefinition;
    pub use super::bodies::TetrisBricks;

    pub use super::level::Level;

    pub use super::assets::GameAssets;
}
