//! Disastris is a game for bevy-jam-2. Here a short overview of the organization of the source code. Besides a list of
//! important ToDos for the next release in this doc string some ideas for a refactoring are given:
//!
//! This file libs.rs provides an entry point for the game with the [`start_disastris`] function. Based
//! on the [`GameConfig`] struct the startup application state (see [`GameState`]) and the loaded level can be
//! configured.
//!
//! Disastris also supports the feature **debug** which adds a [`::bevy_inspector_egui`] to the game.
//!
//! ## TODOs for next Release 0.2.0v
//! * [x] Update to bevy 0.10.0
//! * [x] Do some refactoring on module layer
//! * [ ] Fix WASM build
//! * [ ] Have some transition states between levels
//! * [ ] Make the levels data driven

use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::DefaultPlugins;
use bevy_tweening::TweeningPlugin;

use prelude::*;
use rand::Rng;

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub mod data;
pub mod field;
pub mod game;
pub mod hud;
pub mod input;
pub mod movement;
pub mod render_old;
pub mod state;
pub mod view;

pub const PX_PER_TILE: f32 = 32.0;
pub const SECONDS_PER_ROUND: f32 = 0.5;
pub const PX_PER_ICON: f32 = 64.0;

pub const QUOTE1: &str = "You won and all you get is this damn quote: \"Back in my days they delivered the raw materials in clean 4-block packages!\" by Gereon Bartel - Senior Block Composer";
pub const QUOTE2: &str = "You won and all you get is this damn quote: \"Rearranging ugly blobs into more elegant shapes is part of my daily routine!\" by Tim Janus - Fulltime Code Refactorer";
pub const QUOTE3: &str =
    "You won and all you get is this damn quote: \"look Ma, no javascript!\" by psi - obnoxious Rust fanboy";

pub const TUTORIAL: &str = "Disassemble the useless input blob that was delivered and combine the parts to something beautiful! Select tools and place them in the factory. Choose between several tool variants with the mouse wheel and hit the simulate button when you're ready.";

pub const TUT1: &str = "Hello disastros engineer, your task is to move the gray BLOB such that it hits the red target area. On the right toolbar you see multiple tools - You have one ROTATOR, place it wisely somewhere in the blue building area. Play starts the simlation.";
pub const TUT2: &str = "Well done disastros engineer, your second tasks involves multiple tools. After you selected a tool on the right toolbar you can change its variant via the mouse wheel. Place all tools to the blue building area in a way that the gray BLOB moves into the red target area.";
pub const TUT3: &str = "Yass, lets get disastros and let us apply the CUTTER tool! There are many variants, remember the mouse wheel to select them. Place the cutter and other tools on the blue building area. Do you have what it needs to fill up the red target area?";

pub fn get_random_quote() -> String {
    let v = vec![QUOTE1, QUOTE2, QUOTE3];

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..v.len());
    v[idx].to_string()
}

pub const Z_FIELD: f32 = 0.0;
pub const Z_TRANS: f32 = 10.0;
pub const Z_SOLID: f32 = 20.0;
pub const Z_OVERLAY: f32 = 30.0;

pub mod constants {
    pub use super::Z_FIELD;
    pub use super::Z_OVERLAY;
    pub use super::Z_SOLID;
    pub use super::Z_TRANS;

    pub use super::QUOTE1;
    pub use super::QUOTE2;
    pub use super::QUOTE3;
    pub use super::TUT1;
    pub use super::TUT2;
    pub use super::TUT3;
    pub use super::TUTORIAL;

    pub use super::PX_PER_ICON;
    pub use super::PX_PER_TILE;
    pub use super::SECONDS_PER_ROUND;
}

pub mod prelude {
    pub use crate::data::prelude::*;
    pub use crate::field::prelude::*;
    pub use crate::hud::prelude::*;
    pub use crate::state::prelude::*;
}

/// An enumeration of different SystemSets that are ordered
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum GameSets {
    EventHandling,
    InputAndDispatch,
    GameUpdates,
    RenderUpdates,
}

#[derive(States, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum GameState {
    #[default]
    /// The state is always the first state
    InternalStartup,

    /// The main menu of the game
    Mainmenu,

    /// The ingame state where the actual action happens
    PlayLevel,

    /// Animation test code
    AnimationTest,
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::InternalStartup => write!(f, "InternalStartup"),
            GameState::Mainmenu => write!(f, "Mainmenu"),
            GameState::PlayLevel => write!(f, "PlayLevel"),
            GameState::AnimationTest => write!(f, "AnimationTest"),
        }
    }
}

impl FromStr for GameState {
    type Err = ();

    fn from_str(input: &str) -> Result<GameState, Self::Err> {
        match input.trim().to_lowercase().as_str() {
            // we do not want to convert a string to InternalStartup
            "mainmenu" => Ok(GameState::Mainmenu),
            "playlevel" => Ok(GameState::PlayLevel),
            "animationtest" => Ok(GameState::AnimationTest),
            _ => Err(()),
        }
    }
}

/// Represents the current startup configuration
#[derive(Resource)]
pub struct GameConfig {
    pub start_level: u32,

    pub start_state: String,
}

/// Acts as an entry point for the game.
pub fn start_disastris(config: GameConfig) {
    let mut app = App::new();

    app.insert_resource(config);

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Disastris - A contribution to bevy-jam2".into(),
            position: WindowPosition::Centered(MonitorSelection::Primary),
            resolution: (1400., 1000.).into(),
            present_mode: PresentMode::AutoVsync,
            // Tells wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_plugin(input::InputMappingPlugin)
    .add_plugin(TweeningPlugin);

    app.add_event::<movement::BlobMoveEvent>()
        .add_event::<view::ViewUpdate>();

    app.add_state::<GameState>();

    // initial initializiation during startup
    app.add_startup_system(initial_start_setup);

    // Initialization and cleanup for a disastris level
    app.add_systems((
        game::spawn_world.in_schedule(OnEnter(GameState::PlayLevel)),
        hud::spawn_hud.in_schedule(OnEnter(GameState::PlayLevel)),
        clean_all_system.in_schedule(OnExit(GameState::PlayLevel)),
    ));

    // @TODO initialization and cleanup for other states
    app.add_systems((
        hud::spawn_hud.in_schedule(OnEnter(GameState::Mainmenu)),
        clean_all_system.in_schedule(OnExit(GameState::Mainmenu)),
    ));

    // Setup the game loop
    // 1. We start with a fresh field state and updating the turn resource
    app.add_system(state::progress_level_time_system.before(GameSets::EventHandling));
    app.add_system(field::field_states_generation_system.before(GameSets::EventHandling));
    app.add_system(field::field_states_generation_system.after(GameSets::EventHandling));

    // we can check for win its not important if we realize it one tick later:
    app.add_system(game::level_won_system);

    app.add_systems((
        //animate_rendered_blob_system.in_set(GameSets::EventHandling),
        view::handle_view_update_system.in_set(GameSets::EventHandling),
        field::tool::apply_movement_tools.in_set(GameSets::EventHandling),
        field::tool::apply_cutter_tool.in_set(GameSets::EventHandling),
    ));

    app.add_systems((
        hud::toolbar_button_system.in_set(GameSets::InputAndDispatch),
        input::tool_switch_via_mouse_wheel_system.in_set(GameSets::InputAndDispatch),
        input::grid_coordinate_via_mouse_system.in_set(GameSets::InputAndDispatch),
        movement::move_events_by_gravity_system.in_set(GameSets::InputAndDispatch),
        field::blob::move_blob_by_input.in_set(GameSets::InputAndDispatch),
        //contiously_spawn_tetris_at_end,
    ));

    app.add_systems((
        movement::handle_move_blob_events.in_set(GameSets::GameUpdates),
        input::create_tool_if_valid_clicked.in_set(GameSets::GameUpdates),
    ));

    app.add_systems((
        view::animate_rendered_blob_system.in_set(GameSets::RenderUpdates),
        hud::toolbar_images_system.in_set(GameSets::RenderUpdates),
        hud::toolbar_inventory_system.in_set(GameSets::RenderUpdates),
        hud::toolbar_overlays_system.in_set(GameSets::RenderUpdates),
        render_old::old_render_entities_system::<field::Field>.in_set(GameSets::RenderUpdates), // still needed to render target blob @todo get rid of it
        render_old::show_block_with_debug_tag_system.in_set(GameSets::RenderUpdates),
    ));

    app.configure_set(GameSets::EventHandling.before(GameSets::InputAndDispatch));
    app.configure_set(GameSets::GameUpdates.after(GameSets::InputAndDispatch));

    // Add an ingame inspector window
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_type::<Coordinate>()
        .register_type::<Blob>()
        .register_type::<Block>()
        .register_type::<Target>()
        .register_type::<Field>()
        .register_type::<UITagImage>()
        .register_type::<UITagHover>()
        .register_type::<UITagInventory>()
        .register_type::<Interaction>()
        .register_type::<Level>()
        .register_type::<GameStateLevel>()
        .register_type::<game::RealBlob>()
        .register_type::<PlayerStateLevel>();

    // Setup animation demo
    view::register_animation_demo(&mut app);

    app.run();
}

/// setups global information like the asset structure and the current level
fn initial_start_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
) {
    info!("Create global resources for Assets, Gamestate and ViewConfig");
    // setup the camera
    commands.spawn(Camera2dBundle::default());

    //commands.insert_resource(WinitSettings::desktop_app());
    commands.insert_resource(state::PlayerStateLevel::new());
    commands.insert_resource(data::level::Level::new(config.start_level));
    commands.insert_resource(state::GameStateLevel::new(SECONDS_PER_ROUND));

    let assets = data::assets::GameAssets::new(&asset_server);
    let id = view::spawn_simple_rendering_entity(&mut commands).id();
    commands.insert_resource(view::ViewConfig {
        renderer_entity: id,
        factory_topleft: Vec3::ZERO,
        tetris_topleft: Vec3::ZERO,
        anim_duration: Duration::from_millis(200),
        brick_image: assets.block_blob.clone(),
        test_blob: None,
    });
    commands.insert_resource(assets);

    // Switch state
    let state = GameState::from_str(config.start_state.as_str()).unwrap_or(GameState::PlayLevel);
    next_state.set(state);
    info!(
        "Switching state from '{}' --> '{}'",
        GameState::InternalStartup,
        state
    );
}

pub fn clean_all_system(mut commands: Commands, query: Query<Entity>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
