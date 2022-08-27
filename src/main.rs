use bevy::{prelude::*, window::WindowMode};
use bevy_jam_2_disastris_lib::{game_assets::GameAssets, prelude::*, SECONDS_PER_ROUND};

#[cfg(feature = "debug")]
use {
    bevy_inspector_egui::{
        RegisterInspectable,
        //        InspectorPlugin,
        WorldInspectorPlugin,
    },
    bevy_jam_2_disastris_lib::blob::{Blob, BlobGravity, Coordinate},
    bevy_jam_2_disastris_lib::field::Field,
    bevy_jam_2_disastris_lib::hud::{UITagHover, UITagImage},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum MySystems {
    EventHandling,
    Input,
    GameUpdates,
    RenderUpdates,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        width: 1600.0,
        height: 1000.0,
        position: bevy::window::WindowPosition::Centered(MonitorSelection::Primary),
        title: "Disastris - A contribution to bevy-jam-2".into(),
        resizable: true,
        decorations: true,
        cursor_visible: true,
        cursor_locked: false,
        mode: WindowMode::Windowed,
        transparent: false,
        ..Default::default()
    })
    .insert_resource(Turn::new(SECONDS_PER_ROUND));

    app.add_event::<BlobMoveEvent>()
        .add_event::<BlobTeleportEvent>();

    // Use default pluign and show own plugin for input mapping
    app.add_plugins(DefaultPlugins)
        // the following plugin is an example on how bigger units
        // of functionalities can be structured
        .add_plugin(InputMappingPlugin);

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
                .with_system(progress_turn)
                .with_system(contiously_spawn_tetris_at_end)
                .with_system(remove_field_lines)
                .label(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blob_by_player)
                .with_system(toolbar_button_system)
                .with_system(tool_switch_on_mouse_wheel)
                .with_system(teleport_blob_out_of_factory)
                .label(MySystems::Input)
                .after(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blobs_by_gravity)
                .with_system(move_factory_blobs_by_events)
                .with_system(move_production_blobs_by_events)
                .with_system(move_field_content_down_if_not_occupied)
                .with_system(mouse_for_field_selection_and_tool_creation)
                .label(MySystems::GameUpdates)
                .after(MySystems::Input),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(grid_update_render_entities::<Blob>)
                .with_system(grid_update_render_entities::<Field>)
                .with_system(update_toolbar)
                .with_system(blob_update_transforms)
                .with_system(update_field_debug)
                .label(MySystems::RenderUpdates)
                .after(MySystems::GameUpdates),
        );

    // Add an ingame inspector window
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Coordinate>()
        .register_inspectable::<BlobGravity>()
        .register_inspectable::<Blob>()
        .register_inspectable::<Field>()
        .register_inspectable::<UITagImage>()
        .register_inspectable::<UITagHover>()
        .register_type::<Interaction>();
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<State<GameState>>,
) {
    // setup the camera
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            //    scaling_mode: ScalingMode::FixedVertical(1000.0),
            ..Default::default()
        },
        ..Default::default()
    });

    //commands.insert_resource(WinitSettings::desktop_app());
    commands.insert_resource(GameAssets::new(&asset_server));
    commands.insert_resource(PlayerState::new());
    commands.insert_resource(Level::level_01());

    // Switch state
    app_state.overwrite_set(GameState::Ingame).unwrap();
}
