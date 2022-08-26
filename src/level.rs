use crate::prelude::*;
use bevy::utils::HashMap;

pub struct Level {
    pub start_blob: Vec<i32>,

    pub target_figure: Vec<i32>,

    pub applicable_tools: HashMap<Tool, usize>,
}

impl Level {
    pub fn new() -> Self {
        Level {
            start_blob: prototype::gen_blob_body(),
            target_figure: prototype::gen_target_body(),
            applicable_tools: HashMap::new(),
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}
