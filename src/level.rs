use crate::prelude::*;
use bevy::utils::HashMap;

pub struct Level {
    pub start_blob: (Vec<i32>, (i32, i32)),

    pub target_figure: (Vec<i32>, (i32, i32)),

    pub applicable_tools: HashMap<Tool, usize>,
}

impl Level {
    pub fn new() -> Self {
        Level::level_01()
    }

    pub fn level_01() -> Self {
        let mut applicable_tools = HashMap::new();
        applicable_tools.insert(Tool::Move(MoveDirection::default()), 2);
        applicable_tools.insert(Tool::Rotate(RotateDirection::default()), 1);
        applicable_tools.insert(Tool::Cutter(TetrisBricks::default()), 1);

        Level {
            start_blob: (
                prototype::gen_blob_body(0).expect("Couldn't generate start blob"),
                (3, -4),
            ),
            target_figure: (
                prototype::gen_target_body(1).expect("Couldn't generate target figure"),
                (0, 12),
            ),
            applicable_tools,
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}
