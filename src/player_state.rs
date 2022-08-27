use std::fmt::Display;

use bevy::prelude::Component;
use bevy::reflect::Reflect;
use bevy::utils::HashMap;

use crate::blob::Coordinate;
use crate::bodies::TetrisBricks;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum MoveDirection {
    #[default]
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
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

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum Tool {
    Move(MoveDirection),
    Rotate(RotateDirection),
    Cutter(TetrisBricks),
    #[default]
    Play,
    Stop,
}

impl TryFrom<i32> for Tool {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            101 => Ok(Tool::Move(MoveDirection::Up)),
            102 => Ok(Tool::Move(MoveDirection::Down)),
            103 => Ok(Tool::Move(MoveDirection::Left)),
            104 => Ok(Tool::Move(MoveDirection::Right)),

            201 => Ok(Tool::Rotate(RotateDirection::Left)),
            202 => Ok(Tool::Rotate(RotateDirection::Right)),

            301 => Ok(Tool::Cutter(TetrisBricks::Square)),
            302 => Ok(Tool::Cutter(TetrisBricks::Line)),
            303 => Ok(Tool::Cutter(TetrisBricks::L)),
            304 => Ok(Tool::Cutter(TetrisBricks::InvL)),
            305 => Ok(Tool::Cutter(TetrisBricks::StairsL)),
            306 => Ok(Tool::Cutter(TetrisBricks::StairsR)),
            307 => Ok(Tool::Cutter(TetrisBricks::SmallT)),

            401 => Ok(Tool::Play),

            501 => Ok(Tool::Stop),
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
            Tool::Play => 401,
            Tool::Stop => 501,
        }
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Tool::Move(_) => "Move",
            Tool::Rotate(_) => "Rotate",
            Tool::Cutter(_) => "Cut",
            Tool::Play => "Play",
            Tool::Stop => "Stop",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Component)]
pub struct PlayerState {
    pub selected_tool: Option<Tool>,

    pub applicable_tools: HashMap<Tool, usize>,

    pub tool_placement_coordinate: Option<Coordinate>,
}

impl PlayerState {
    pub fn new() -> PlayerState {
        let mut applicable_tools = HashMap::new();
        applicable_tools.insert(Tool::Rotate(RotateDirection::Left), 1);

        PlayerState {
            selected_tool: None,
            applicable_tools,
            tool_placement_coordinate: None,
        }
    }
}
