use bevy::prelude::Component;
use bevy::reflect::Reflect;

use crate::bodies::TetrisBricks;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum MoveDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum RotateDirection {
    #[default]
    Left,
    Right,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum Tool {
    Direction(MoveDirection),
    Rotate(RotateDirection),
    Cutter(TetrisBricks),
    #[default]
    Play,
    Stop,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct PlayerState {
    pub selected_tool: Option<Tool>,
}
