use crate::data::prelude::*;
use crate::input::add_tetris_control;
use crate::render_old::RenderableGrid;
use crate::state::GameState;
use crate::SECONDS_PER_ROUND;
use crate::{field::spawn_field, prelude::*};
use bevy::{log, prelude::*};

use crate::view::prelude::*;

use crate::hud::spawn_text;

use crate::field::blob::spawn_blob_from_body_definition;
use crate::field::target::spawn_target;

#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct RealBlob {}

pub fn spawn_world(
    mut commands: Commands, // stores commands for entity/component creation / deletion
    assets: Res<GameAssets>, // used to access files stored in the assets folder.
    mut gs: ResMut<GameState>,
    mut player_state: ResMut<PlayerStateLevel>,
    mut view_config: ResMut<ViewConfig>,
    mut evt: EventWriter<ViewUpdate>,
) {
    let level = gs.get_lvl();
    info!("Spawn world for level '{}' called.", level.num);
    player_state.set_inventory(level.applicable_tools.clone());

    commands.insert_resource(GameStateLevel::new(SECONDS_PER_ROUND));

    let factory_field_struct = Field::as_factory();
    let root_factory_field = Vec3::new(0., -70., 0.0);
    let (px, py) = factory_field_struct.coords_to_px(0, 0);
    view_config.factory_topleft = Vec3::new(px, py, 0.0) + root_factory_field;

    let fac_field_id = spawn_field(
        &mut commands,
        &assets,
        factory_field_struct,
        "Factory Field",
        root_factory_field,
    );
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
    info!("Send ViewUpdate::BlobSpawned for Start Blob!");

    let _target_stone = spawn_target(
        &mut commands,
        level.target_figure.0.clone(),
        "Target Stone",
        Some(level.target_figure.1.into()),
        &|_| {},
    );
    //evt.send(ViewUpdate::BlobSpawned(target_stone));

    spawn_text(
        &mut commands,
        &assets,
        level.get_text(),
        Vec2::new(-492., 188.),
        Vec2::new(408., 512.),
        Color::WHITE,
        Color::rgba(0.0, 0.0, 0.75, 0.75),
    );
}

pub fn contiously_spawn_tetris_at_end(
    mut commands: Commands,
    query_active: Query<&Blob>,
    query_field: Query<Entity, With<Field>>,
    mut level_state: ResMut<GameStateLevel>,
) {
    if let Ok(prod_ent) = query_field.get_single() {
        if level_state.is_new_turn() && query_active.iter().filter(|g| g.active).count() == 0 {
            let body = gen_random_tetris_body();

            let _new_id = spawn_blob_from_body_definition(
                &mut commands,
                BodyDefinition::as_blob(body),
                format!(
                    "{}. Additional Tetris Brick",
                    level_state.num_additional_bricks
                )
                .as_str(),
                prod_ent,
                IVec2 { x: -3, y: 3 },
                &|ec| {
                    add_tetris_control(ec);
                    ec.insert(RealBlob {});
                },
            );
            level_state.num_additional_bricks += 1;
            unimplemented!("continiously spawn tetris at end needs to send events for renderer");
            //commands.entity(prod_ent).push_children(&[new_id]);
        }
    } else {
        panic!("The programmer forgot to create a Field and so no spawning of tetris bricks...");
    }
}

pub fn level_won_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query_target: Query<&Target>,
    mut query_field: Query<&mut Field>,
    mut player_state: ResMut<PlayerStateLevel>,
) {
    if player_state.won {
        return;
    }
    //~

    let target = if let Ok(target) = query_target.get_single() {
        target
    } else {
        return;
    };

    let field = if let Ok(field) = query_field.get_single_mut() {
        field
    } else {
        return;
    };
    //~

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

        spawn_text(
            &mut commands,
            &assets,
            "YOU WON!!!\n\nPress <RETURN> to continue!\n\nAnd get a huge wall of text with your random quote as a reward!",
            Vec2::new(-450., -300.),
            Vec2::new(512., 386.),
            Color::WHITE,
            Color::BLACK,
        );
    }
}
