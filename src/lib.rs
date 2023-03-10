//! Disastris is a game for bevy-jam-2. Here a short overview of the organization of the source code with some
//! ideas for a refactoring is given:
//!
//! This file libs.rs provides an entry point for the game with the [`start_disastris`] function. In the function
//! the bevy scheduler is setup (atm a 0.9.1 version of the scheduler is used)
//!
//! Disastris also supports the feature **Debug** which atm adds a [`::bevy_inspector_egui`] to the game.
//!
//!
//! # Refactoring of modules
//! @todo Overwork the modules
//!
//! * [x] The modules [`field::blob`], [`field::Block`], [`field::tool`] and [`field::target`], are used to describe entites that are on the field  
//! Suggestion: Merge them in the module [`field`] but allow qualification without sub module name
//! * [ ] Also move [`field_element`] into the [`field`] module and clarify that it is Cache of the current field
//! * [ ] The modules [`bodies`], [`level`] and [`game_assets`] contain data that is serializable, e.g. levels that may
//! be stored on the hard drive or bodies that describe different shapes, e.g for tetris stones.
//! Suggestion: Add a *data* module and summarize the elements there.
//! * [ ] The modules [`player_state`] and [`turn`] contain elements of the game state.
//! Suggestion: Move them into a module *state* and also move [`game::GameState`] there.
//! * [ ] Update to latest bevy engine version and overwork the scheduler in [`start_disastris`]
//!

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_tweening::TweeningPlugin;

#[cfg(feature = "debug")]
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

use prelude::field::*;
use prelude::*;
use rand::Rng;

pub mod bodies;
pub mod field;
pub mod field_element;
pub mod game;
pub mod game_assets;
pub mod hud;
pub mod input;
pub mod level;
pub mod movement;
pub mod player_state;
pub mod render_old;
pub mod turn;
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

pub mod prelude {
    pub use crate::bodies::*;
    pub use crate::field::prelude::*;
    pub use crate::field_element::*;
    pub use crate::game::*;
    pub use crate::game_assets::*;
    pub use crate::hud::*;
    pub use crate::input::*;
    pub use crate::level::*;
    pub use crate::movement::*;
    pub use crate::player_state::*;
    pub use crate::render_old::*;
    pub use crate::tool::*;
    pub use crate::turn::*;
    pub use crate::view::*;

    pub use crate::*;
}

/// An enumeration of different Systems that are ordered
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum MySystems {
    EventHandling,
    Input,
    PreGameUpdates,
    GameUpdates,
    PostGameUpdates,
    RenderUpdates,
}

#[derive(Resource)]
pub struct GameConfig {
    start_level: u32,
}

/// Acts as an entry point for the game.
pub fn start_disastris(_resource_folder: PathBuf, level: u32) {
    let config = GameConfig { start_level: level };

    let mut app = App::new();

    app.insert_resource(config);

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            width: 1600.0,
            height: 1000.0,
            position: WindowPosition::Centered,
            title: "Disastris - A contribution to bevy-jam-2".into(),
            resizable: true,
            decorations: true,
            cursor_visible: true,
            mode: WindowMode::Windowed,
            transparent: false,
            ..default()
        },
        ..default()
    }))
    .add_plugin(InputMappingPlugin)
    .add_plugin(TweeningPlugin);

    app.add_event::<BlobMoveEvent>();

    // Setup the game loop
    app.add_state(GameState::Starting)
        // a SystemSet allows easier state management but comes with
        // pitfals for fixed-time run criterias, see:
        // https://bevy-cheatbook.github.io/programming/states.html
        .add_system_set(SystemSet::on_enter(GameState::Starting).with_system(setup))
        .add_system_set(
            SystemSet::on_enter(GameState::Ingame)
                .with_system(spawn_world)
                .with_system(spawn_hud),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(progress_turn_system)
                //.with_system(contiously_spawn_tetris_at_end)
                //.with_system(remove_field_lines)
                .with_system(animate_rendered_blob_system)
                .with_system(handle_view_update_system)
                .with_system(field_states_generation_system)
                .label(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blob_by_player)
                .with_system(toolbar_button_system)
                .with_system(tool_switch_via_mouse_wheel_system)
                .with_system(grid_coordinate_via_mouse_system)
                .label(MySystems::Input)
                .after(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(apply_movement_tools)
                .with_system(apply_cutter_tool)
                .label(MySystems::PreGameUpdates)
                .after(MySystems::Input),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_events_by_gravity_system)
                .with_system(field_states_generation_system)
                .label(MySystems::GameUpdates)
                .after(MySystems::PreGameUpdates),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blobs_by_events_system)
                .with_system(tool_creation_via_mouse_system)
                .label(MySystems::PostGameUpdates)
                .after(MySystems::GameUpdates),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(win_condition_system)
                .with_system(old_render_entities_system::<Field>)
                .with_system(toolbar_images_system)
                .with_system(toolbar_inventory_system)
                .with_system(toolbar_overlays_system)
                .with_system(show_block_with_debug_tag_system)
                .label(MySystems::RenderUpdates)
                .after(MySystems::PostGameUpdates),
        )
        .add_system_set(SystemSet::on_exit(GameState::Ingame).with_system(clean_all_system));

    // Add an ingame inspector window
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Coordinate>()
        .register_inspectable::<Blob>()
        .register_inspectable::<Block>()
        .register_inspectable::<Target>()
        .register_inspectable::<Field>()
        .register_inspectable::<UITagImage>()
        .register_inspectable::<UITagHover>()
        .register_inspectable::<UITagInventory>()
        .register_type::<Interaction>();

    // Setup animation demo
    register_animation_demo(&mut app, GameState::AnimationTest);

    app.run();
}

/// setups global information like the asset structure and the current level
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<State<GameState>>,
    config: Res<GameConfig>,
) {
    // setup the camera
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            //    scaling_mode: ScalingMode::FixedVertical(1000.0),
            ..Default::default()
        },
        ..Default::default()
    });

    //commands.insert_resource(WinitSettings::desktop_app());
    commands.insert_resource(PlayerState::new());
    commands.insert_resource(Level::new(config.start_level));
    commands.insert_resource(Turn::new(SECONDS_PER_ROUND));

    let assets = GameAssets::new(&asset_server);
    commands.insert_resource(assets);

    // Switch state
    app_state.overwrite_set(GameState::Ingame).unwrap();
}

pub fn clean_all_system(mut commands: Commands, query: Query<Entity>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
