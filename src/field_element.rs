use bevy::prelude::*;
use crate::prelude::*;

// chat log form psi architecture / refactor discussion

/// Encapsules the game state of a game field.
pub struct FactoryFieldState {
    dimensions: IVec2,
    /// Field elements.
    elements: Vec<FieldElement>,
}

pub enum FieldElementKind {
    Block,
    Blob,
    Move,
    Rotate,
    Cut(TetrisBricks),
}

pub struct FieldElement {
    entity: Entity,
    blob: Option<Entity>,
    kind: FieldElementKind,
    position: IVec2,
}