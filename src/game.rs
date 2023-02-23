use std::time::Duration;

use crate::{blob::*, field::spawn_field, game_assets::GameAssets, prelude::*, target::*};
use bevy::{log, prelude::*};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameState {
    /// The startup for loading stuff etc.
    Starting,

    /// The ingame state where the actual action happens!
    Ingame,

    /// Animation test code
    AnimationTest,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct RealBlob {}

pub fn spawn_world(
    mut commands: Commands, // stores commands for entity/component creation / deletion
    assets: Res<GameAssets>, // used to access files stored in the assets folder.
    mut turn: ResMut<Turn>,
    level: Res<Level>,
    mut player_state: ResMut<PlayerState>,
    mut evt: EventWriter<ViewUpdate>,
) {
    player_state.applicable_tools = level.applicable_tools.clone();

    let factory_field_struct = Field::as_factory(&assets);
    let root_factory_field = Vec3::new(-200.0, -70.0, 0.0);
    let (px, py) = factory_field_struct.coords_to_px(0, 0);
    let origin_factory = Vec3::new(px, py, 0.0) + root_factory_field;

    let fac_field_id = spawn_field(
        &mut commands,
        &assets,
        factory_field_struct,
        "Factory Field",
        root_factory_field,
        &|ec| {
            ec.insert(FactoryFieldTag {});
        },
    );
    turn.fac_id = Some(fac_field_id);
    log::info!("Factory field spawned with id: {:?}", fac_field_id);

    let start_blob = spawn_blob_from_body_definition(
        &mut commands,
        BodyDefinition::as_blob(level.start_blob.0.clone()),
        "Start Blob",
        fac_field_id,
        level.start_blob.1.into(),
        &|ec| {
            #[cfg(feature = "debug")]
            add_tetris_control(ec);

            ec.insert(RealBlob {});
        },
    );
    evt.send(ViewUpdate::BlobSpawned(start_blob));

    let id = spawn_simple_rendering_entity(&mut commands).id();
    commands.insert_resource(ViewConfig {
        renderer_entity: id,
        factory_topleft: origin_factory,
        tetris_topleft: Vec3::ZERO,
        anim_duration: Duration::from_millis(200),
        brick_image: assets.block_blob.clone(),
        test_blob: None,
    });

    let _target_stone = spawn_target(
        &mut commands,
        &assets.block_target_outline,
        level.target_figure.0.clone(),
        "Target Stone",
        Some(level.target_figure.1.into()),
        &|_| {},
    );
    //evt.send(ViewUpdate::BlobSpawned(target_stone));

    let pos = UiRect {
        top: Val::Percent(3.0),
        left: Val::Percent(3.0),
        ..default()
    };
    spawn_text(&mut commands, &assets, TUTORIAL, pos, &|_| {});
}

pub fn contiously_spawn_tetris_at_end(
    mut commands: Commands,
    query_active: Query<&Blob>,
    mut turn: ResMut<Turn>,
) {
    if let Some(prod_ent) = turn.prod_id {
        if turn.is_new_turn() && query_active.iter().filter(|g| g.active).count() == 0 {
            let body = gen_random_tetris_body();

            let _new_id = spawn_blob_from_body_definition(
                &mut commands,
                BodyDefinition::as_blob(body),
                format!("{}. Additional Tetris Brick", turn.num_additional_bricks).as_str(),
                prod_ent,
                IVec2 { x: -3, y: 3 },
                &|ec| {
                    add_tetris_control(ec);
                    ec.insert(RealBlob {});
                },
            );
            turn.num_additional_bricks += 1;
            unimplemented!("continiously spawn tetris at end needs to send events for renderer");
            //commands.entity(prod_ent).push_children(&[new_id]);
        }
    } else {
        panic!("The programmer forgot to create the production Field...");
    }
}

pub fn check_win(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query_target: Query<&Target>,
    mut query_field: Query<&mut Field, With<FactoryFieldTag>>,
    mut player_state: ResMut<PlayerState>,
) {
    if player_state.won {
        return;
    }
    //~

    let target = query_target.single();
    let field = query_field.single_mut();
    let field_state = field.get_field_state();

    let coords = target.occupied_coordinates();
    let transformed_coords = coords
        .iter()
        .map(|(c, r)| IVec2::new(*c, *r)) // as the factory field 12 tiles larger in y direction
        .collect();

    let cond = field_state.are_all_coordinates(
        &transformed_coords,
        None,
        &|el| el.kind == FieldElementKind::Block(None), // none means the blob is not existing and the blocks are direcly linked to the field
    )
        //&& coords.len() == field.num_occupied()
        ;
    if cond {
        player_state.won = true;
        let pos = UiRect {
            top: Val::Percent(3.0),
            right: Val::Percent(3.0),
            ..default()
        };
        spawn_text(&mut commands, &assets, &get_random_quote(), pos, &|_| {});
    }
}
