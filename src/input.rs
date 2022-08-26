use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, input::mouse::MouseWheel, log, prelude::*};
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

pub fn tool_switch_on_mouse_wheel(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player_state: ResMut<PlayerState>,
) {
    for event in mouse_wheel_events.iter() {
        let y = event.y.signum() as i32;
        if let Some(tool) = &mut player_state.selected_tool {
            match tool {
                Tool::Move(d) => {
                    let mut cur = *d as i32;
                    cur += y;
                    if cur < 1 {
                        cur = MoveDirection::max();
                    } else if cur > MoveDirection::max() {
                        cur = 1;
                    }
                    *tool = Tool::Move(cur.try_into().unwrap_or_else(|_| {
                        panic!("Error in Enum Trait try_from({})<MoveDirection>", cur);
                    }));
                }
                Tool::Rotate(d) => {
                    let mut cur = *d as i32;
                    cur += y;
                    if cur < 1 {
                        cur = RotateDirection::max();
                    } else if cur > RotateDirection::max() {
                        cur = 1;
                    }
                    *tool = Tool::Rotate(cur.try_into().unwrap_or_else(|_| {
                        panic!("Error in Enum Trait try_from({})<RotateDirection>", cur);
                    }));
                }
                Tool::Cutter(blob) => {
                    let mut cur = *blob as i32;
                    cur += y;
                    cur = cur.clamp(0, TetrisBricks::max());
                    if cur < 1 {
                        cur = TetrisBricks::max();
                    } else if cur > TetrisBricks::max() {
                        cur = 1;
                    }
                    *tool = Tool::Cutter(cur.try_into().unwrap_or_else(|_| {
                        panic!("Error in Enum Trait try_from({})<TetrisBricks>", cur);
                    }));
                }
                _ => {}
            }
        }
    }
}

pub fn mouse_for_field_selection_and_tool_creation(
    windows: Res<Windows>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut sprites: Query<(&mut Sprite, &GlobalTransform, &Coordinate, &Parent), With<FieldRenderTag>>,
    mut field_query: Query<(Entity, &mut Field), With<FactoryFieldTag>>,
    mut player_state: ResMut<PlayerState>,
) {
    let (field_id, mut field) = field_query.single_mut();

    let ev = cursor_moved_events.iter().last();
    if let (Some(moved), Some(window)) = (ev, windows.get_primary()) {
        let half_window = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        let cursor_pos = moved.position - half_window;
        player_state.tool_placement_coordinate = None;

        for (mut sprite, trans, coord, parent) in sprites.iter_mut() {
            // only continue when hovering a sprite on the factory field
            if field_id != parent.get() {
                continue;
            }
            //~

            let sprite_pos = trans.translation();
            let diff = Vec3::new(
                sprite_pos.x - cursor_pos.x,
                sprite_pos.y - cursor_pos.y,
                0.0,
            );

            // sprite is a cube so x test is enough
            if diff.length() < (PX_PER_TILE / 2.0) {
                sprite.color = Color::WHITE;
                let (x, y) = (coord.c, coord.r);
                log::info!("Mouse over: Coordinate {x},{y}");
                player_state.tool_placement_coordinate = Some(*coord);
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let (Some(tool), Some(coord)) = (
            player_state.selected_tool,
            player_state.tool_placement_coordinate,
        ) {
            if tool != Tool::Play && tool != Tool::Stop {
                log::info!("Place tool {:?} at ({},{})", tool, coord.c, coord.r);
                field.mutate_at_coordinate((coord.c, coord.r), &move |field, _, idx| {
                    field
                        .set_occupied(idx, Into::<i32>::into(tool))
                        .expect("wrong coordinate in set_occupied due to mouse click");
                })
            }
        }
    }
}
