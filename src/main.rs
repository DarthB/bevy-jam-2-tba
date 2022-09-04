use std::time::Duration;

use bevy::{prelude::*, window::WindowMode};
use bevy_jam_2_disastris_lib::prelude::*;
use bevy_tweening::TweeningPlugin;

#[cfg(feature = "debug")]
use {
    bevy_inspector_egui::{
        RegisterInspectable,
        //        InspectorPlugin,
        WorldInspectorPlugin,
    },
    bevy_jam_2_disastris_lib::block::Block,
    bevy_jam_2_disastris_lib::blob::Blob,
    bevy_jam_2_disastris_lib::target::{Coordinate, Target},
    bevy_jam_2_disastris_lib::field::Field,
    bevy_jam_2_disastris_lib::hud::{UITagHover, UITagImage, UITagInventory},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum MySystems {
    EventHandling,
    Input,
    PreGameUpdates,
    GameUpdates,
    PostGameUpdates,
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
    });

    app.add_event::<BlobMoveEvent>()
        .add_event::<BlobTeleportEvent>();

    // Use default pluign and show own plugin for input mapping
    app.add_plugins(DefaultPlugins)
        // the following plugin is an example on how bigger units
        // of functionalities can be structured
        .add_plugin(InputMappingPlugin)
        .add_plugin(TweeningPlugin);

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
                //                .with_system(contiously_spawn_tetris_at_end)
                //.with_system(remove_field_lines)
                .with_system(stupid_block_update)
                .with_system(crate::view::handle_view_updates)
                .label(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blob_by_player)
                .with_system(toolbar_button_system)
                .with_system(tool_switch_on_mouse_wheel)
                .label(MySystems::Input)
                .after(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(generate_field_states)
                .with_system(generate_move_events_by_gravity)
                .label(MySystems::PreGameUpdates)
                .after(MySystems::Input)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_factory_blobs_by_events)
                .with_system(move_production_blobs_by_events)
                .with_system(mouse_for_field_selection_and_tool_creation)
                .label(MySystems::GameUpdates)
                .after(MySystems::PreGameUpdates),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)            
                .with_system(teleport_blob_out_of_factory)
                .with_system(generate_field_states)
                .label(MySystems::PostGameUpdates)
                .after(MySystems::GameUpdates)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(check_win)
                .with_system(grid_update_render_entities::<Field>)
                .with_system(update_toolbar_images)
                .with_system(update_toolbar_inventory)
                .with_system(update_toolbar_overlays)
                .with_system(blob_update_transforms)
                .with_system(update_field_debug)
                .label(MySystems::RenderUpdates)
                .after(MySystems::PostGameUpdates),
        )
        .add_system_set(SystemSet::on_exit(GameState::Ingame).with_system(clean_all));

    // Add an ingame inspector window
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Coordinate>()
        .register_inspectable::<Blob>()
        .register_inspectable::<Block>()
        .register_inspectable::<Coordinate>()
        .register_inspectable::<Target>()
        .register_inspectable::<Field>()
        .register_inspectable::<FactoryFieldTag>()
        .register_inspectable::<ProductionFieldTag>()
        .register_inspectable::<FieldRenderTag>()
        .register_inspectable::<FieldState>()
        .register_inspectable::<FieldElement>()
        .register_inspectable::<FieldElementKind>()
        .register_inspectable::<UITagImage>()
        .register_inspectable::<UITagHover>()
        .register_inspectable::<UITagInventory>()
        .register_type::<Interaction>();

    // Setup animation demo
    crate::view::register_animation_demo(&mut app, GameState::AnimationTest);

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
    commands.insert_resource(PlayerState::new());
    commands.insert_resource(Level::level_01());
    commands.insert_resource(Turn::new(SECONDS_PER_ROUND));

    let assets = GameAssets::new(&asset_server);
    commands.insert_resource(assets);

    // Switch state
    app_state.overwrite_set(GameState::Ingame).unwrap();
}

pub fn clean_all(mut commands: Commands, query: Query<Entity>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
