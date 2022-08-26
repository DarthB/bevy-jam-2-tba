//! # Module for presentation and animations
//!
//!
//!

use crate::{
    blob::{Blob, Coordinate},
    game_assets::GameAssets,
    input::TetrisActionsWASD,
    PX_PER_TILE,
};
use bevy::prelude::*;
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

//----------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Rotation {
    Left,
    Right,
}

pub enum ViewUpdate {
    /// A new blob has been spawned in the factory
    BlobSpawned(Entity),
    /// A blob has been moved
    BlobMoved(Entity),
    /// A blob has been rotated
    BlobRotated(Entity, Rotation),
    /// A blog has been transferred from the factory to tetris arena
    BlobTransferred(Entity),
}

/// Configuration struct for integration into the rest of the project
pub struct ViewConfig {
    /// The factory field entity
    factory: Entity,
    /// The tetris field entity
    tetris: Entity,

    brick_image: Handle<Image>,

    test_blob: Option<Entity>,
}

//----------------------------------------------------------------------

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct BlobRenderState {
    root: Entity,
    last_trans: Vec3,
}

//----------------------------------------------------------------------

fn coord_to_translation(coord: Coordinate) -> Vec3 {
    Vec3::new(
        -coord.c as f32 * PX_PER_TILE - PX_PER_TILE / 2.0,
        -coord.r as f32 * PX_PER_TILE - PX_PER_TILE / 2.0,
        0.0,
    )
}

fn handle_blob_spawned(
    commands: &mut Commands,
    blob: Entity,
    blobs: &Query<&Blob>,
    config: &Res<ViewConfig>,
) {
    if let Ok(blobdata) = blobs.get(blob) {
        let trans = coord_to_translation(blobdata.coordinate.unwrap_or_default());
        let root = commands
            .spawn_bundle(SpatialBundle::from(Transform::from_translation(trans)))
            .with_children(|cb| {
                for r in 0..Blob::size() {
                    for c in 0..Blob::size() {
                        if blobdata.body[Blob::coords_to_idx(r, c)] != 0 {
                            cb.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::WHITE,
                                    custom_size: Some(Vec2::ONE * PX_PER_TILE),
                                    ..Default::default()
                                },
                                transform: Transform::from_xyz(
                                    c as f32 * -PX_PER_TILE,
                                    r as f32 * -PX_PER_TILE,
                                    0.0,
                                ),
                                texture: config.brick_image.clone(),
                                ..Default::default()
                            });
                        }
                    }
                }
            })
            .id();
        commands.entity(config.factory).add_child(root);
        commands.entity(blob).insert(BlobRenderState {
            root,
            last_trans: trans,
        });
    }
}

fn handle_blob_moved(
    commands: &mut Commands,
    blob: Entity,
    blobs: &mut Query<(&Blob, &mut BlobRenderState)>,
    transforms: &mut Query<&mut Transform>,
    config: &Res<ViewConfig>,
) {
    if let Ok((blobdata, mut state)) = blobs.get_mut(blob) {
        let last_trans = state.last_trans;
        let trans = coord_to_translation(blobdata.coordinate.unwrap_or_default());

        if let Ok(mut x) = transforms.get_mut(state.root) {
            x.translation = trans;
        }
        state.last_trans = trans;
    }
}

pub fn handle_view_updates(
    mut commands: Commands,
    mut ev: EventReader<ViewUpdate>,
    blobs: Query<&Blob>,
    mut rendered_blobs: Query<(&Blob, &mut BlobRenderState)>,
    mut transforms: Query<&mut Transform>,
    config: Res<ViewConfig>,
) {
    for ev in ev.iter() {
        match *ev {
            ViewUpdate::BlobSpawned(blob) => {
                handle_blob_spawned(&mut commands, blob, &blobs, &config)
            }
            ViewUpdate::BlobMoved(blob) => handle_blob_moved(
                &mut commands,
                blob,
                &mut rendered_blobs,
                &mut transforms,
                &config,
            ),
            _ => unimplemented!(),
        }
    }
}

//----------------------------------------------------------------------

pub fn setup_demo_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut evt: EventWriter<ViewUpdate>,
) {
    let factory = commands
        .spawn_bundle::<SpatialBundle>(Transform::from_xyz(20.0, 30.0, 0.0).into())
        .with_children(|cb| {
            cb.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::ONE * PX_PER_TILE * 8.0),
                    ..Default::default()
                },
                transform: Transform::from_xyz(-PX_PER_TILE * 4.0, -PX_PER_TILE * 4.0, 0.0),
                ..Default::default()
            });
        })
        .insert(Name::new("Mock Factoryfield"))
        .insert_bundle(InputManagerBundle::<TetrisActionsWASD> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new([
                (KeyCode::W, TetrisActionsWASD::Up),
                (KeyCode::S, TetrisActionsWASD::Down),
                (KeyCode::A, TetrisActionsWASD::Left),
                (KeyCode::D, TetrisActionsWASD::Right),
                (KeyCode::Q, TetrisActionsWASD::LRotate),
                (KeyCode::E, TetrisActionsWASD::RRotate),
            ]),
        })
        .id();

    let tetris = commands.spawn().insert(Name::new("Mock Tetrisfield")).id();

    let blob = commands
        .spawn()
        .insert(Blob {
            body: crate::bodies::prototype::gen_target_body2(),
            coordinate: Some(Coordinate { c: 1, r: -4 }),
            ..Default::default()
        })
        .insert(Name::new("Test Blob"))
        .id();

    commands.insert_resource(ViewConfig {
        factory,
        tetris,
        brick_image: assets.block_blob.clone(),
        test_blob: Some(blob),
    });

    evt.send(ViewUpdate::BlobSpawned(blob));
}

pub fn demo_system(
    mut commands: Commands,
    config: Res<ViewConfig>,
    mut blobs: Query<&mut Blob>,
    query: Query<&ActionState<TetrisActionsWASD>>,
    mut evt: EventWriter<ViewUpdate>,
) {
    query.for_each(|s| {
        if s.just_pressed(TetrisActionsWASD::Down) {
            bevy::log::info!("DOWN pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut blobdata) = blobs.get_mut(test_blob) {
                    let mut coord = blobdata.coordinate.unwrap_or_default();
                    coord.r += 1;
                    blobdata.coordinate = Some(coord);
                    evt.send(ViewUpdate::BlobMoved(test_blob));
                }
            }
        }
    });
}
