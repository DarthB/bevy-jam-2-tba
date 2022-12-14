use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

use std::fmt::Display;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, FromReflect)]
pub enum MoveDirection {
    #[default]
    Up = 1,
    Right = 2,
    Down = 3,
    Left = 4,
}

impl TryFrom<i32> for MoveDirection {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == MoveDirection::Up as i32 => Ok(MoveDirection::Up),
            x if x == MoveDirection::Down as i32 => Ok(MoveDirection::Down),
            x if x == MoveDirection::Left as i32 => Ok(MoveDirection::Left),
            x if x == MoveDirection::Right as i32 => Ok(MoveDirection::Right),
            _ => Err(()),
        }
    }
}

impl MoveDirection {
    pub fn min() -> i32 {
        1
    }

    pub fn max() -> i32 {
        4
    }
}

impl From<MoveDirection> for (i32, i32) {
    fn from(d: MoveDirection) -> Self {
        match d {
            MoveDirection::Up => (0, -1),
            MoveDirection::Down => (0, 1),
            MoveDirection::Left => (-1, 0),
            MoveDirection::Right => (1, 0),
        }
    }
}

impl From<MoveDirection> for IVec2 {
    fn from(d: MoveDirection) -> Self {
        match d {
            MoveDirection::Up => IVec2 { x: 0, y: -1 },
            MoveDirection::Right => IVec2 { x: 1, y: 0 },
            MoveDirection::Down => IVec2 { x: 0, y: 1 },
            MoveDirection::Left => IVec2 { x: -1, y: 0 },
        }
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, FromReflect)]
pub enum RotateDirection {
    #[default]
    Left = 1,
    Right = 2,
}

impl TryFrom<i32> for RotateDirection {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == RotateDirection::Left as i32 => Ok(RotateDirection::Left),
            x if x == RotateDirection::Right as i32 => Ok(RotateDirection::Right),
            _ => Err(()),
        }
    }
}

impl RotateDirection {
    pub fn min() -> i32 {
        1
    }

    pub fn max() -> i32 {
        2
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, Component)]
pub enum Tool {
    Move(MoveDirection),
    Rotate(RotateDirection),
    Cutter(TetrisBricks),
    #[default]
    Simulate,
    Reset,
    Eraser,
    EraseAll,
}

impl Tool {
    pub fn as_default_variant(self) -> Self {
        match self {
            Tool::Move(_) => Tool::Move(MoveDirection::default()),
            Tool::Rotate(_) => Tool::Rotate(RotateDirection::default()),
            Tool::Cutter(_) => Tool::Cutter(TetrisBricks::default()),
            _ => self,
        }
    }
}

impl TryFrom<i32> for Tool {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            101 => Ok(Tool::Move(MoveDirection::Up)),
            102 => Ok(Tool::Move(MoveDirection::Right)),
            103 => Ok(Tool::Move(MoveDirection::Down)),
            104 => Ok(Tool::Move(MoveDirection::Left)),

            201 => Ok(Tool::Rotate(RotateDirection::Left)),
            202 => Ok(Tool::Rotate(RotateDirection::Right)),

            301 => Ok(Tool::Cutter(TetrisBricks::Square)),
            302 => Ok(Tool::Cutter(TetrisBricks::Line)),
            303 => Ok(Tool::Cutter(TetrisBricks::L)),
            304 => Ok(Tool::Cutter(TetrisBricks::InvL)),
            305 => Ok(Tool::Cutter(TetrisBricks::StairsL)),
            306 => Ok(Tool::Cutter(TetrisBricks::StairsR)),
            307 => Ok(Tool::Cutter(TetrisBricks::SmallT)),

            401 => Ok(Tool::Simulate),

            501 => Ok(Tool::Reset),

            601 => Ok(Tool::Eraser),

            701 => Ok(Tool::EraseAll),
            _ => Err(()),
        }
    }
}

impl From<Tool> for i32 {
    fn from(t: Tool) -> Self {
        match t {
            Tool::Move(d) => 100 + d as i32,
            Tool::Rotate(d) => 200 + d as i32,
            Tool::Cutter(brick) => 300 + brick as i32,
            Tool::Simulate => 401,
            Tool::Reset => 501,
            Tool::Eraser => 601,
            Tool::EraseAll => 701,
        }
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Tool::Move(_) => "Move",
            Tool::Rotate(_) => "Rotate",
            Tool::Cutter(_) => "Cut",
            Tool::Simulate => "Play",
            Tool::Reset => "Pause",
            Tool::Eraser => "Eraser",
            Tool::EraseAll => "Reset Factory",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Component, Resource)]
pub struct PlayerState {
    pub selected_tool: Option<Tool>,

    pub applicable_tools: HashMap<Tool, usize>,

    pub tool_placement_coordinate: Option<IVec2>,

    pub won: bool,
}

impl PlayerState {
    pub fn new() -> PlayerState {
        let applicable_tools = HashMap::new();

        PlayerState {
            selected_tool: None,
            applicable_tools,
            tool_placement_coordinate: None,
            won: false,
        }
    }

    pub fn num_in_inventory(&self, tool: Tool) -> usize {
        // ensure default variants are used
        let tool = tool.as_default_variant();

        if let Some(num) = self.applicable_tools.get(&tool) {
            *num
        } else {
            usize::MAX
        }
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
