use crate::{blob::*, field::spawn_field, game_assets::GameAssets, prelude::*};
use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameState {
    /// The startup for loading stuff etc.
    Starting,

    /// The ingame state where the actual action happens!
    Ingame,
}

pub fn spawn_world(
    mut commands: Commands, // stores commands for entity/component creation / deletion
    assets: Res<GameAssets>,
    mut turn: ResMut<Turn>, // used to access files stored in the assets folder.
) {
    let comp = Field::as_factory();
    let fac_field = spawn_field(
        &mut commands,
        &assets.factory_floor,
        comp,
        "Factory Field",
        Vec3::new(-350.0, 0.0, 0.0),
        &|ec| {
            ec.insert(FactoryFieldTag {});
        },
    );
    turn.fac_id = Some(fac_field);

    let l_stone = spawn_blob(
        &mut commands,
        &assets,
        bodies::prototype::gen_blob_body2(),
        "L Stone",
        Some(Coordinate { c: 3, r: -4 }),
        &|ec| {
            add_tetris_control(ec);
        },
    );
    commands.entity(fac_field).push_children(&[l_stone]);

    let pr_field = spawn_field(
        &mut commands,
        &assets.tetris_floor,
        Field::as_production_field(),
        "Production Field",
        Vec3::new(480.0, 0.0, 0.0),
        &|ec| {
            ec.insert(ProductionFieldTag {});
        },
    );
    turn.prod_id = Some(pr_field);

    let t_stone = spawn_blob(
        &mut commands,
        &assets,
        bodies::prototype::gen_target_body2(),
        "Target Stone",
        Some(Coordinate { c: 4, r: -3 }),
        &|_| {},
    );
    commands.entity(pr_field).push_children(&[t_stone]);
}

pub fn contiously_spawn_tetris_at_end(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query_active: Query<&BlobGravity>,
    mut turn: ResMut<Turn>,
) {
    if let Some(prod_ent) = turn.prod_id {
        if turn.is_new_turn() && query_active.iter().filter(|g| g.active).count() == 0 {
            let body = gen_random_tetris_body();

            let new_id = spawn_blob(
                &mut commands,
                &assets,
                body,
                format!("{}. Additional Tetris Brick", turn.num_additional_bricks).as_str(),
                Some(Coordinate { r: -3, c: 3 }),
                &|ec| {
                    add_tetris_control(ec);
                },
            );
            turn.num_additional_bricks += 1;
            commands.entity(prod_ent).push_children(&[new_id]);
        }
    } else {
        panic!("The programmer forgot to create the production Field...");
    }
}
