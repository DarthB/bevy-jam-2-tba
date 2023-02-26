//! This crate defines the Disastris binaries. Disastris is a contribution to bevy-jam-2.
//! Most of its functionality and therefore the documentation relies in the [`::bevy_jam_2_disastris_lib`].

use bevy::{prelude::*, window::WindowMode};
use bevy_jam_2_disastris_lib::{
    field::{blob::move_blob_by_player, field_states_generation_system, tool::apply_cutter_tool},
    prelude::{
        field_selection_via_mouse_system, move_blobs_by_events_system,
        move_events_by_gravity_system, spawn_hud, spawn_world, teleport_event_system,
        tool_creation_via_mouse_system, tool_switch_via_mouse_wheel_system, toolbar_button_system,
        toolbar_images_system, toolbar_inventory_system, toolbar_overlays_system,
        win_condition_system, BlobMoveEvent, BlobTeleportEvent, Field, GameAssets, GameState,
        InputMappingPlugin, Level, PlayerState,
    },
    render_old::{old_render_entities_system, show_block_with_debug_tag_system},
    turn::{progress_turn_system, Turn},
    view::{animate_rendered_blob_system, handle_view_update_system, register_animation_demo},
    SECONDS_PER_ROUND,
};
use bevy_tweening::TweeningPlugin;

#[cfg(feature = "debug")]
use {
    bevy_inspector_egui::{
        RegisterInspectable,
        //        InspectorPlugin,
        WorldInspectorPlugin,
    },
    bevy_jam_2_disastris_lib::blob::Blob,
    bevy_jam_2_disastris_lib::block::Block,
    bevy_jam_2_disastris_lib::field::Field,
    bevy_jam_2_disastris_lib::hud::{UITagHover, UITagImage, UITagInventory},
    bevy_jam_2_disastris_lib::target::{Coordinate, Target},
};

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

/// Setups a bevy app object and adds the default plugins, systems and events
fn main() {
    let mut app = App::new();
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

    app.add_event::<BlobMoveEvent>()
        .add_event::<BlobTeleportEvent>();

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
                //                .with_system(contiously_spawn_tetris_at_end)
                //.with_system(remove_field_lines)
                .with_system(animate_rendered_blob_system)
                .with_system(handle_view_update_system)
                .label(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blob_by_player)
                .with_system(toolbar_button_system)
                .with_system(tool_switch_via_mouse_wheel_system)
                .label(MySystems::Input)
                .after(MySystems::EventHandling),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(field_states_generation_system)
                .with_system(move_events_by_gravity_system)
                .label(MySystems::PreGameUpdates)
                .after(MySystems::Input),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(move_blobs_by_events_system)
                .with_system(field_selection_via_mouse_system)
                .label(MySystems::GameUpdates)
                .after(MySystems::PreGameUpdates),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ingame)
                .with_system(teleport_event_system)
                .with_system(field_states_generation_system)
                .with_system(tool_creation_via_mouse_system)
                .with_system(apply_cutter_tool)
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
        .register_inspectable::<Coordinate>()
        .register_inspectable::<Target>()
        .register_inspectable::<Field>()
        .register_inspectable::<FactoryFieldTag>()
        .register_inspectable::<ProductionFieldTag>()
        .register_inspectable::<FieldRenderTag>()
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
    commands.insert_resource(Level::level_01());
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
