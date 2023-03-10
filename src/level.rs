use crate::prelude::*;
use bevy::{prelude::Resource, utils::HashMap};

#[derive(Resource)]
pub struct Level {
    pub start_blob: (Vec<i32>, (i32, i32)),

    pub target_figure: (Vec<i32>, (i32, i32)),

    pub applicable_tools: HashMap<Tool, usize>,

    level_text: String,
}

impl Level {
    pub fn get_text(&self) -> &str {
        &self.level_text
    }

    pub fn new(num: u32) -> Self {
        match num {
            1 => Self::level_01(),
            2 => Self::level_02(),
            3 => Self::level_03(),
            _ => panic!("Level '{}' not supported yet", num),
        }
    }

    pub fn level_01() -> Self {
        let mut applicable_tools = HashMap::new();
        applicable_tools.insert(Tool::Rotate(RotateDirection::default()), 1);
        applicable_tools.insert(Tool::Move(MoveDirection::default()), 0);
        applicable_tools.insert(Tool::Cutter(TetrisBricks::default()), 0);
        Self::level_helper(1, applicable_tools)
    }

    pub fn level_02() -> Self {
        let mut applicable_tools = HashMap::new();
        applicable_tools.insert(Tool::Move(MoveDirection::default()), 2);
        applicable_tools.insert(Tool::Rotate(RotateDirection::default()), 1);
        applicable_tools.insert(Tool::Cutter(TetrisBricks::default()), 0);
        Self::level_helper(2, applicable_tools)
    }

    pub fn level_03() -> Self {
        let mut applicable_tools = HashMap::new();
        applicable_tools.insert(Tool::Move(MoveDirection::default()), 0);
        applicable_tools.insert(Tool::Rotate(RotateDirection::default()), 1);
        applicable_tools.insert(Tool::Cutter(TetrisBricks::default()), 1);
        Self::level_helper(3, applicable_tools)
    }

    fn level_helper(num: u32, applicable_tools: HashMap<Tool, usize>) -> Self {
        let lvl_txt = match num {
            1 => TUT1,
            2 => TUT2,
            3 => TUT3,
            _ => "NO TEXT FOR LEVEL yet",
        };

        Level {
            start_blob: (
                prototype::gen_blob_body(num).expect("Couldn't generate start blob"),
                (3, -4),
            ),
            target_figure: (
                prototype::gen_target_body(num).expect("Couldn't generate target figure"),
                (0, 12),
            ),
            applicable_tools,
            level_text: lvl_txt.to_owned(),
        }
    }
}
