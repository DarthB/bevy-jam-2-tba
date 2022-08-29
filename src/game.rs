use crate::{blob::*, field::spawn_field, game_assets::GameAssets, prelude::*, target::*};
use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameState {
    /// The startup for loading stuff etc.
    Starting,

    /// The ingame state where the actual action happens!
    Ingame,
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
) {
    player_state.applicable_tools = level.applicable_tools.clone();

    let comp = Field::as_factory(&assets);
    let fac_field = spawn_field(
        &mut commands,
        &assets,
        comp,
        "Factory Field",
        Vec3::new(-200.0, 0.0, 0.0),
        &|ec| {
            ec.insert(FactoryFieldTag {});
        },
    );
    turn.fac_id = Some(fac_field);

    let l_stone = spawn_blob(
        &mut commands,
        &assets.block_blob,
        &assets,
        level.start_blob.0.clone(),
        "L Stone",
        Some(level.start_blob.1.into()),
        &|ec| {
            #[cfg(feature = "debug")]
            add_tetris_control(ec);

            ec.insert(RealBlob {});
            ec.insert(BlobGravity {
                gravity: (0, 1),
                active: true,
            });
        },
    );
    commands.entity(fac_field).push_children(&[l_stone]);

    let pr_field = spawn_field(
        &mut commands,
        &assets,
        Field::as_production_field(&assets),
        "Production Field",
        Vec3::new(300.0, 0.0, 0.0),
        &|ec| {
            ec.insert(ProductionFieldTag {});
        },
    );
    turn.prod_id = Some(pr_field);

    let t_stone = spawn_target(
        &mut commands,
        &assets.block_target_outline,
        &assets,
        level.target_figure.0.clone(),
        "Target Stone",
        Some(level.target_figure.1.into()),
        &|_| {},
    );
    commands.entity(pr_field).push_children(&[t_stone]);

    let pos = UiRect {
        top: Val::Percent(3.0),
        left: Val::Percent(3.0),
        ..default()
    };
    spawn_text(&mut commands, &assets, TUTORIAL, pos, &|_| {});
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
                &assets.block_blob,
                &assets,
                body,
                format!("{}. Additional Tetris Brick", turn.num_additional_bricks).as_str(),
                Some(Coordinate { r: -3, c: 3 }),
                &|ec| {
                    add_tetris_control(ec);
                    ec.insert(RealBlob {});
                    ec.insert(BlobGravity {
                        gravity: (0, 1),
                        active: true,
                    });
                },
            );
            turn.num_additional_bricks += 1;
            commands.entity(prod_ent).push_children(&[new_id]);
        }
    } else {
        panic!("The programmer forgot to create the production Field...");
    }
}

pub fn check_win(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query_field: Query<&Field, With<ProductionFieldTag>>,
    query_target: Query<&Target>,
    mut player_state: ResMut<PlayerState>,
) {
    if player_state.won {
        return;
    }

    let target = query_target.single();
    let field = query_field.single();

    let mut coords = target.occupied_coordinates();
    coords = coords
        .iter()
        .map(|(c, r)| (*c - 10, *r - 6 as i32))
        .collect();
    let cond = field.all_coordinates_occupied(&coords, false);
    if coords.len() == field.num_occupied() && cond {
        player_state.won = true;
        let pos = UiRect {
            top: Val::Percent(3.0),
            right: Val::Percent(3.0),
            ..default()
        };
        spawn_text(&mut commands, &assets, &get_random_quote(), pos, &|_| {});
    }
}
