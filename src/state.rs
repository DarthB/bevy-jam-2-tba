use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

pub mod prelude {
    pub use super::GameStateLevel;
    pub use super::PlayerStateLevel;

    pub use super::progress_level_time_system;
}

#[derive(Default, Resource, Reflect)]
pub struct GameStateLevel {
    cur_time: f32,

    new_turn: bool,

    time_per_turn: f32,

    num_turn: i32,

    pub doing_simulation: bool,

    pub num_additional_bricks: i32,
}

impl GameStateLevel {
    pub fn new(time_per_turn: f32) -> Self {
        GameStateLevel {
            cur_time: 0.0,
            time_per_turn,
            new_turn: false,
            num_turn: 0,
            num_additional_bricks: 0,
            doing_simulation: true,
        }
    }

    pub fn apply_time(&mut self, dt: f32) {
        if self.doing_simulation {
            return;
        }
        //~

        self.new_turn = false;
        self.cur_time += dt;
        if self.cur_time > self.time_per_turn {
            self.cur_time -= self.time_per_turn;
            self.new_turn = true;
            self.num_turn += 1;
        }
    }

    pub fn is_new_turn(&self) -> bool {
        self.new_turn && !self.doing_simulation
    }
    pub fn get_num_turn(&self) -> i32 {
        self.num_turn
    }
}

/// Contains the current state of the player during a level, e.g. its selected tool and a tool inventory
#[derive(Debug, Clone, Default, PartialEq, Eq, Component, Resource, Reflect)]
pub struct PlayerStateLevel {
    /// the currently selected tool
    pub selected_tool: Option<Tool>,

    /// the number of applicable tools, i.e. the inventory of tools
    applicable_tools: HashMap<Tool, usize>,

    /// A coordinate that stores where a tool shall be placed. Not yet used.
    pub tool_placement_coordinate: Option<IVec2>,

    /// A flag indicating of the player has won the level
    pub won: bool,
}

impl PlayerStateLevel {
    pub fn new() -> PlayerStateLevel {
        let applicable_tools = HashMap::new();

        PlayerStateLevel {
            selected_tool: None,
            applicable_tools,
            tool_placement_coordinate: None,
            won: false,
        }
    }

    pub fn set_inventory(&mut self, new_inventory: HashMap<Tool, usize>) {
        self.applicable_tools = new_inventory;
    }

    ///
    pub fn num_in_inventory(&self, tool: Tool) -> Option<usize> {
        // ensure default variants are used
        let tool = tool.as_default_variant();
        self.applicable_tools.get(&tool).copied()
    }

    pub fn add_to_inventory(&mut self, tool: Tool, change: i32) -> bool {
        // ensure default variants are used
        let tool = tool.as_default_variant();

        if let Some(num) = self.applicable_tools.get(&tool) {
            let res = *num as i32 + change;
            if res < 0 {
                return false;
            }
            let inv_num = self.applicable_tools.entry(tool).or_insert(0);
            *inv_num = res as usize;
        } else if change > 0 {
            self.applicable_tools.insert(tool, change as usize);
        }

        true
    }
}

pub fn progress_level_time_system(mut level_state: ResMut<GameStateLevel>, time: ResMut<Time>) {
    level_state.apply_time(time.delta_seconds());
}
