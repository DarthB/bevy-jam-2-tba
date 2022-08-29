use crate::{blob::*, field::spawn_field, game_assets::GameAssets, prelude::*, target::*};
use bevy::prelude::*;

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

    let comp = Field::as_factory(&assets);
    let fac_field = spawn_field(
        &mut commands,
        &assets,
        comp,
        "Factory Field",
        Vec3::new(120.0, 64.0, 0.0),
        turn.use_old_rendering,
        &|ec| {
            ec.insert(FactoryFieldTag {});
        },
    );
    turn.fac_id = Some(fac_field);

    let start_blob = spawn_blob(
        &mut commands,
        &assets.block_blob,
        &assets,
        BlobBody::new(level.start_blob.0.clone()),
        "L Stone",
        level.start_blob.1.into(),
        turn.use_old_rendering,
        &|ec| {
            #[cfg(feature = "debug")]
            add_tetris_control(ec);

            ec.insert(RealBlob {});
        },
    );
    evt.send(ViewUpdate::BlobSpawned(start_blob));
    commands.entity(fac_field).push_children(&[start_blob]);

    let pr_field = spawn_field(
        &mut commands,
        &assets,
        Field::as_production_field(&assets),
        "Production Field",
        Vec3::new(600.0, 0.0, 0.0),
        turn.use_old_rendering,
        &|ec| {
            ec.insert(ProductionFieldTag {});
        },
    );
    turn.prod_id = Some(pr_field);

    let target_stone = spawn_target(
        &mut commands,
        &assets.block_target_outline,
        &assets,
        level.target_figure.0.clone(),
        "Target Stone",
        Some(level.target_figure.1.into()),
        &|_| {},
    );
    evt.send(ViewUpdate::BlobSpawned(target_stone));
    //commands.entity(pr_field).push_children(&[target_stone]);

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
    query_active: Query<&Blob>,
    mut turn: ResMut<Turn>,
) {
    if let Some(prod_ent) = turn.prod_id {
        if turn.is_new_turn() && query_active.iter().filter(|g| g.active).count() == 0 {
            let body = gen_random_tetris_body();

            let new_id = spawn_blob(
                &mut commands,
                &assets.block_blob,
                &assets,
                BlobBody::new(body),
                format!("{}. Additional Tetris Brick", turn.num_additional_bricks).as_str(),
                IVec2 { x: -3, y: 3 },
                turn.use_old_rendering,
                &|ec| {
                    add_tetris_control(ec);
                    ec.insert(RealBlob {});
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
    mut query_field: Query<&mut Field, With<ProductionFieldTag>>,
    query_target: Query<&Target>,
    mut player_state: ResMut<PlayerState>,
) {
    if player_state.won {
        return;
    }

    let target = query_target.single();
    let field = query_field.single_mut();
    let field_state = field.get_field_state();

    let coords = target.occupied_coordinates();
    let transformed_coords = coords
        .iter()
        .map(|(c, r)| IVec2::new(*c, *r + 6)) // as the production field 6 tiles larger in y direction
        .collect();

    let cond = field_state.are_all_coordinates_occupied(
        &transformed_coords, 
        None, 
        &|el| el.kind != FieldElementKind::Empty,
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
