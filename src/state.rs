use crate::get_random_quote;
use crate::{hud::spawn_text, prelude::*, DisastrisAppState};

use bevy::{prelude::*, utils::HashMap};

use crate::movement::prelude::*;

use crate::data::prelude::*;

pub mod prelude {
    pub use super::GameStateLevel;
    pub use super::PlayerStateLevel;

    pub use super::progress_level_time_system;
}

#[derive(Debug, Default, Reflect, Clone, Copy, Eq, PartialEq, Hash, Component)]
pub struct OverStatePersistenceTag {}

#[derive(Default, Resource, Reflect, FromReflect)]
pub struct GameState {
    pub level: Option<Level>,

    pub upcoming_level: Option<Level>,
}

impl GameState {
    /// gets a reference to the current level. If there is no level it switches to the upcoming level
    pub fn get_lvl(&mut self) -> &Level {
        if self.level.is_none() {
            self.next_lvl();
        }

        self.level.as_ref().unwrap()
    }

    /// moves to the upcoming level
    pub fn next_lvl(&mut self) {
        self.level = self.upcoming_level.take();
        self.upcoming_level = None;
    }
}

/// Stores the game state of the current running level (puzzle)
#[derive(Default, Resource, Reflect)]
pub struct GameStateLevel {
    /// the current time spend in the level
    cur_time: f32,

    /// a flag that is ture if this tick indicates a new turn, such that movement can be triggered
    new_turn: bool,

    time_per_turn: f32,

    num_turn: i32,

    pub simulation_running: bool,

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
            simulation_running: true,
        }
    }

    pub fn apply_time(&mut self, dt: f32) {
        if self.simulation_running {
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
        self.new_turn && !self.simulation_running
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

pub fn app_state_transition_system(
    cur_state: Res<State<DisastrisAppState>>,
    mut next_state: ResMut<NextState<DisastrisAppState>>,
    mut gamestate: ResMut<GameState>,
    mut playerstate: ResMut<PlayerStateLevel>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        let did_transition = match cur_state.0 {
            DisastrisAppState::PlayLevel => {
                if playerstate.won {
                    let next_num = gamestate.level.as_ref().unwrap().num + 1;
                    playerstate.won = false;

                    if next_num <= 3 {
                        gamestate.upcoming_level = Some(crate::data::level::Level::new(next_num));
                    } else {
                        gamestate.upcoming_level = None;
                    }
                    next_state.set(DisastrisAppState::TransitionLevel);
                    true
                } else {
                    false
                }
            }
            DisastrisAppState::TransitionLevel => {
                if gamestate.upcoming_level.is_none() {
                    next_state.set(DisastrisAppState::Mainmenu);
                } else {
                    gamestate.next_lvl();
                    next_state.set(DisastrisAppState::PlayLevel);
                }
                true
            }
            _ => {
                warn!(
                    "No App State transition from {} yet (triggered by <RETURN>)",
                    cur_state.0.to_string()
                );
                false
            }
        };

        if did_transition {
            info!(
                "State transition {} --> {}",
                cur_state.0,
                next_state.0.as_ref().unwrap()
            );
        }
    }
}

pub fn spawn_transition_level(mut commands: Commands, assets: Res<GameAssets>, gs: Res<GameState>) {
    if let Some(level) = &gs.level {
        // this is a finished level or the last level

        spawn_text(
            &mut commands,
            &assets,
            &format!("Congratulations for finishing level '{}'!", level.num),
            UiRect {
                left: Val::Percent(5.),
                right: Val::Percent(45.),
                top: Val::Percent(0.),
                bottom: Val::Percent(10.),
            },
            None,
        );

        spawn_text(
            &mut commands,
            &assets,
            &format!(
                "Your reward quote:\n{}\n\nTo start the next level just press return!",
                get_random_quote(),
            ),
            UiRect {
                left: Val::Percent(5.),
                right: Val::Percent(45.),
                top: Val::Percent(20.),
                bottom: Val::Percent(50.),
            },
            Some(Size {
                width: Val::Px(600.0),
                height: Val::Undefined,
            }),
        );

        spawn_text(
            &mut commands,
            &assets,
            "Some Stats:\nTodo: \nTodo: \nTodo:",
            UiRect {
                left: Val::Percent(5.),
                right: Val::Percent(45.),
                top: Val::Percent(75.),
                bottom: Val::Percent(100.),
            },
            Some(Size {
                width: Val::Px(500.0),
                height: Val::Undefined,
            }),
        );
    } else {
        spawn_text(
            &mut commands,
            &assets,
            "Welcome to the level transition screen, the left side is reserved for things you finished. And as you can see it is quite empty. But there is a lot to come!",
            UiRect {
                left: Val::Percent(5.),
                right: Val::Percent(45.),
                top: Val::Percent(0.),
                bottom: Val::Percent(10.),
            },
            None,
        );
    }

    if let Some(level) = &gs.upcoming_level {
        spawn_text(
            &mut commands,
            &assets,
            &format!("Next level will be {}!", level.num),
            UiRect {
                left: Val::Percent(60.),
                right: Val::Percent(80.),
                top: Val::Percent(0.),
                bottom: Val::Percent(10.),
            },
            None,
        );

        spawn_text(
            &mut commands,
            &assets,
            &format!(
                "A Hint:\n{}\n\nTo start the next level just press return!",
                level.get_text()
            ),
            UiRect {
                left: Val::Percent(55.),
                right: Val::Percent(90.),
                top: Val::Percent(20.),
                bottom: Val::Percent(50.),
            },
            Some(Size {
                width: Val::Px(600.0),
                height: Val::Undefined,
            }),
        );

        spawn_text(
            &mut commands,
            &assets,
            &format!(
                "Your Inventory:\nMovers: {}\nRotators: {}\nCutters: {}",
                level
                    .applicable_tools
                    .get(&Tool::Move(MoveDirection::default()))
                    .unwrap_or(&0usize),
                level
                    .applicable_tools
                    .get(&Tool::Rotate(RotateDirection::default()))
                    .unwrap_or(&0usize),
                level
                    .applicable_tools
                    .get(&Tool::Cutter(TetrisBricks::default()))
                    .unwrap_or(&0usize),
            ),
            UiRect {
                left: Val::Percent(55.),
                right: Val::Percent(90.),
                top: Val::Percent(75.),
                bottom: Val::Percent(100.),
            },
            Some(Size {
                width: Val::Px(500.0),
                height: Val::Undefined,
            }),
        );
    } else {
        spawn_text(
            &mut commands,
            &assets,
            "There is no next level, you won hell yeah!",
            UiRect {
                left: Val::Percent(55.),
                right: Val::Percent(80.),
                top: Val::Percent(0.),
                bottom: Val::Percent(10.),
            },
            None,
        );
    }
}

pub fn clean_all_state_entities(
    mut commands: Commands,
    query: Query<Entity, Without<OverStatePersistenceTag>>,
    view_config: Res<crate::view::ViewConfig>,
) {
    for e in query.iter() {
        if e == view_config.renderer_entity {
            continue;
        }
        commands.entity(e).despawn();
    }
}
