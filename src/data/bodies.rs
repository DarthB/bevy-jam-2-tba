//! The bodies are represented by a RxC (row cross column) i32 vector. Hereby the first R elements represent the top row of the body.
//! ATM only 0 and 1 are used to indicate if the given position is solid or not.
//! The tetris stones and [`crate::field::blob::Blob`] objects use 9x9 vectors. The target shape is a 12x10 vector.

use bevy::{prelude::IVec2, reflect::Reflect};
use rand::Rng;

/// Defines a body that is used on a grid. It can describe the body of a [`crate::field::blob::Blob`] or
/// a [`crate::field::target::Target`]
#[derive(Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct BodyDefinition {
    pub block_positions: Vec<i32>,

    pub size: (usize, usize),

    pub pivot: (usize, usize),
}

impl BodyDefinition {
    pub fn as_blob(positions: Vec<i32>) -> Self {
        BodyDefinition {
            block_positions: positions,
            size: (9, 9),
            pivot: (4, 4),
        }
    }

    pub fn get_relative_positions(&self) -> Vec<IVec2> {
        let mut reval = vec![];

        for idx in 0..self.block_positions.len() {
            let num = self.block_positions[idx];
            if num == 0 {
                continue;
            }
            //~

            let x = (idx as i32 % self.size.0 as i32) - self.pivot.0 as i32;
            let y = (idx as i32 / self.size.1 as i32) - self.pivot.1 as i32;

            reval.push(IVec2 { x, y });
        }

        reval
    }
}

/// Describes the 7 default tetris bricks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum TetrisBricks {
    #[default]
    Square = 1,
    Line = 2,
    L = 3,
    InvL = 4,
    StairsL = 5,
    StairsR = 6,
    SmallT = 7,
}

impl TetrisBricks {
    pub fn min() -> i32 {
        1
    }

    pub fn max() -> i32 {
        7
    }
}

impl TryFrom<i32> for TetrisBricks {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == TetrisBricks::Square as i32 => Ok(TetrisBricks::Square),
            x if x == TetrisBricks::Line as i32 => Ok(TetrisBricks::Line),
            x if x == TetrisBricks::L as i32 => Ok(TetrisBricks::L),
            x if x == TetrisBricks::InvL as i32 => Ok(TetrisBricks::InvL),
            x if x == TetrisBricks::StairsL as i32 => Ok(TetrisBricks::StairsL),
            x if x == TetrisBricks::StairsR as i32 => Ok(TetrisBricks::StairsR),
            x if x == TetrisBricks::SmallT as i32 => Ok(TetrisBricks::SmallT),
            _ => Err(()),
        }
    }
}

/// generates a random tetris body
pub fn gen_random_tetris_body() -> Vec<i32> {
    let mut rng = rand::thread_rng();
    let kind = rng.gen_range(TetrisBricks::min()..TetrisBricks::max() + 1);
    gen_tetris_body(
        kind.try_into()
            .expect("The random range is bigger as the given stones, fix this coder!"),
    )
}

/// Generates the body of a specific tetris brick.
///
/// # Arguments
/// * `kind` - A value of the [`TetrisBricks`] enum to indicate which body.
pub fn gen_tetris_body(kind: TetrisBricks) -> Vec<i32> {
    match kind {
        TetrisBricks::Square => gen_square_body(),
        TetrisBricks::Line => gen_line_body(),
        TetrisBricks::L => gen_l_body(),
        TetrisBricks::InvL => gen_inv_l_body(),
        TetrisBricks::StairsL => gen_stairs_l_body(),
        TetrisBricks::StairsR => gen_stairs_r_body(),
        TetrisBricks::SmallT => gen_t_body(),
    }
}

/// generates a square tetris brick as i32 9x9 flag vector
pub fn gen_square_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// generates a line tetris brick as i32 9x9 flag vector
pub fn gen_line_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// generates a L tetris brick as i32 9x9 flag vector
pub fn gen_l_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// generates an inverse L tetris brick as i32 9x9 flag vector
pub fn gen_inv_l_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 1, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// generates a stairs from left to right tetris brick as i32 9x9 flag vector
pub fn gen_stairs_l_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 1, 0, 0, 0, //
        0, 0, 0, 1, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// generates a stairs from right to left tetris brick as i32 9x9 flag vector
pub fn gen_stairs_r_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 1, 1, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// generates T tetris brick as i32 9x9 flag vector
pub fn gen_t_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 1, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

/// The module contains prototypical level generation methods for start blobs and target areas

/// generate a 9x9 i32 flag vector that represents the starting blob given the level number
pub fn gen_blob_body(level: u32) -> Result<Vec<i32>, String> {
    match level {
        0 => Ok(vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, //
        ]),
        1 | 2 | 3 => Ok(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 1, 0, 0, //
            0, 0, 0, 0, 1, 1, 1, 1, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]),
        4 => Ok(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 1, 0, 1, 1, 0, 0, 0, //
            0, 0, 1, 1, 1, 1, 0, 0, 0, //
            0, 0, 0, 0, 1, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]),
        _ => Err(format!("Invalid Level number: {}", level)),
    }
}

/// generates a 12x10 i32 flag vector that represents the target area
pub fn gen_target_body(level: u32) -> Result<Vec<i32>, String> {
    match level {
        1 => Ok(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //coordinates_of
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, //
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, //
            0, 0, 1, 1, 1, 0, 0, 0, 0, 0, //
            0, 1, 1, 1, 1, 0, 0, 0, 0, 0, //
        ]),
        2 => Ok(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //coordinates_of
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, //
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, //
            0, 0, 0, 0, 0, 0, 0, 1, 1, 1, //
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, //
        ]),
        3 => Ok(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //coordinates_of
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 1, 1, 1, 1, 0, 0, 0, 0, //
            0, 0, 1, 0, 1, 0, 0, 0, 0, 0, //
        ]),
        0 => Ok(vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
        ]),
        _ => Err(format!("Invalid Level number: {}", level)),
    }
}
