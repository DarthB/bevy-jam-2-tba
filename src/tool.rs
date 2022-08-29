use bevy::prelude::*;
use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct ToolComponent {
    pub kind: Tool,

    pub relative_positions: Option<Vec<IVec2>>,
}

impl Default for ToolComponent {
    fn default() -> Self {
        Self { 
            kind: Tool::Move(MoveDirection::Down), 
            relative_positions: Default::default() 
        }
    }
}