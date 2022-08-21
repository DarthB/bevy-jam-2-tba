use bevy::{ecs::system::EntityCommands, prelude::*};
use leafwing_input_manager::prelude::*;

pub struct InputMappingPlugin;

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum WASDActions {
    Left,
    Right,
    Up,
    Down,
    Powerup,
}

/// This is the list of things the player can do in a normal Tetris game
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum TetrisActionsWASD {
    Left,
    Right,
    Up,
    Down,
    LRotate,
    RRotate,
}

impl Plugin for InputMappingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<WASDActions>::default())
            .add_plugin(InputManagerPlugin::<TetrisActionsWASD>::default());
    }
}

pub fn add_tetris_control(commands: &mut EntityCommands) {
    commands.insert_bundle(InputManagerBundle::<TetrisActionsWASD> {
        // Stores "which actions are currently pressed"
        action_state: ActionState::default(),
        // Describes how to convert from player inputs into those actions
        input_map: InputMap::new([
            (KeyCode::W, TetrisActionsWASD::Up),
            (KeyCode::S, TetrisActionsWASD::Down),
            (KeyCode::A, TetrisActionsWASD::Left),
            (KeyCode::D, TetrisActionsWASD::Right),
            (KeyCode::Q, TetrisActionsWASD::LRotate),
            (KeyCode::E, TetrisActionsWASD::RRotate),
        ]),
    });
}

pub fn add_wasd_control(commands: &mut EntityCommands) {
    commands.insert_bundle(InputManagerBundle::<WASDActions> {
        // Stores "which actions are currently pressed"
        action_state: ActionState::default(),
        // Describes how to convert from player inputs into those actions
        input_map: InputMap::new([
            (KeyCode::W, WASDActions::Up),
            (KeyCode::S, WASDActions::Down),
            (KeyCode::A, WASDActions::Left),
            (KeyCode::D, WASDActions::Right),
            (KeyCode::F, WASDActions::Powerup),
        ]),
    });
}

pub fn add_arrow_control(commands: &mut EntityCommands) {
    commands.insert_bundle(InputManagerBundle::<WASDActions> {
        // Stores "which actions are currently pressed"
        action_state: ActionState::default(),
        // Describes how to convert from player inputs into those actions
        input_map: InputMap::new([
            (KeyCode::Up, WASDActions::Up),
            (KeyCode::Down, WASDActions::Down),
            (KeyCode::Left, WASDActions::Left),
            (KeyCode::Right, WASDActions::Right),
            (KeyCode::RShift, WASDActions::Powerup),
        ]),
    });
}
