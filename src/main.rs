use bevy::{prelude::*, window::WindowMode};
use bevy_jam_2_tba_lib::{prelude::*, blob::{move_blob_by_player, blob_update_sprites}, SECONDS_PER_ROUND};

#[cfg(feature = "debug")]
use {
    bevy_inspector_egui:: {
//        InspectorPlugin,
        WorldInspectorPlugin,
        RegisterInspectable
    },

    bevy_jam_2_tba_lib::blob::{Coordinate, BlobGravity},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum MySystems {
    TurnManagement,
    Input,
    GameUpdates,
}

fn main() {
    let mut app = App::new();
    app
        .insert_resource(WindowDescriptor { 
            width: 1366.0, 
            height: 768.0, 
            position: bevy::window::WindowPosition::Centered(MonitorSelection::Primary),
            title: "bevy_jam_2_tba".into(), 
            resizable: true, 
            decorations: true, 
            cursor_visible: true, 
            cursor_locked: false, 
            mode: WindowMode::Windowed, 
            transparent: false, 
            ..Default::default()
    })
        .insert_resource(Turn::new(SECONDS_PER_ROUND))
    ;

    // Use default pluign and show own plugin for input mapping
    app.add_plugins(DefaultPlugins)
        // the following plugin is an example on how bigger units 
        // of functionalities can be structured
        .add_plugin(InputMappingPlugin);

    // Setup the game loop
    app
        .add_state(GameState::Starting)
        // a SystemSet allows easier state management but comes with 
        // pitfals for fixed-time run criterias, see:
        // https://bevy-cheatbook.github.io/programming/states.html
        .add_system_set(
            SystemSet::on_enter(GameState::Starting)
                .with_system(setup)
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Ingame)
                .with_system(spawn_world)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(progress_turn)
                .label(MySystems::TurnManagement)
            )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_players_by_actions)
                .with_system(apply_powerups_by_actions)
                .with_system(move_blob_by_player)
                .label(MySystems::Input)
                .after(MySystems::TurnManagement)    
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blobs_by_gravity)
                .with_system(blob_update_sprites)
                .label(MySystems::GameUpdates)
                .after(MySystems::Input)
        )
    ;

    // Add an ingame inspector window 
    #[cfg(feature = "debug")]
    app
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<UpgradeableMover>()
        .register_inspectable::<Coordinate>()
        .register_inspectable::<BlobGravity>()
    ;

    app.run();
}

fn setup(
    mut commands: Commands, 
    mut app_state: ResMut<State<GameState>>,
) {
    // setup the camera
    commands.spawn_bundle(Camera2dBundle::default());

    // @todo Preload assets

    // Switch state
    app_state.overwrite_set(GameState::Ingame).unwrap();
}
