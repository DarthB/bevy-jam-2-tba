//! # Module for presentation and animations
//!
//!
//!

use std::time::Duration;

use crate::{blob::Blob, game_assets::GameAssets, input::TetrisActionsWASD, PX_PER_TILE};
use bevy::{ecs::system::EntityCommands, prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_tweening::{
    lens::{SpriteColorLens, TransformPositionLens, TransformRotateZLens},
    Animator, EaseFunction, Tween, TweeningType,
};
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

//----------------------------------------------------------------------
// Components I need from the game logic. These *Extra components should
// later be merged into (or replaced with) the game logic components.

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct BlobExtra {
    #[cfg_attr(feature = "debug", inspectable(ignore))]
    /// Blob's blocks. TODO: Currently redundant with parent-child stuff
    blocks: Vec<Entity>,
    /// Position of the Blob's pivot element within the field
    pivot: IVec2,
    /// Transferred: is it on the tetris field?
    transferred: bool,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct BlockExtra {
    /// Position relative to its Blob's pivot
    coordinate: IVec2,
}

//----------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum Rotation {
    Left,
    Right,
}

/// Events for the Renderer
pub enum ViewUpdate {
    /// A new blob has been spawned in the factory
    BlobSpawned(Entity),
    /// A blob has been moved
    BlobMoved(Entity),
    /// A blob has been rotated
    BlobRotated(Entity, Rotation),
    /// A blob has been transferred from the factory to tetris arena
    BlobTransferred(Entity),
    /// A line of blocks was removed in the tetris field.
    LineRemove(Vec<Entity>),
}

/// Configuration struct for integration into the rest of the project
pub struct ViewConfig {
    /// Global renderer entity just for making the entity tree a bit cleaner.
    /// Create a basic one via `spawn_simple_rendering_entity`.
    renderer_entity: Entity,
    /// Global translation to the factory fields's (0,0) block
    factory_topleft: Vec3,
    /// Global translation to the tetris fields's (0,0) block
    tetris_topleft: Vec3,
    /// Animation duration
    anim_duration: Duration,

    brick_image: Handle<Image>,

    /// Used by the demo system (can be ignored)
    test_blob: Option<Entity>,
}

pub fn spawn_simple_rendering_entity<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
) -> EntityCommands<'w, 's, 'a> {
    let mut rv = commands.spawn_bundle(SpatialBundle::default());
    rv.insert(Name::new("Rendering"));
    rv
}

//----------------------------------------------------------------------
// Internal helper components for rendering

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct BlobRenderState {
    /// Last known `pivot` from the game logic
    last_pivot: IVec2,
    /// Cumulative left rotations (0..3)
    rotation_steps: i32,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct BlockRenderState {
    /// Root entity of the block visualization
    root: Entity,
}

//----------------------------------------------------------------------
// utility functions

fn coord_to_translation(coord: IVec2) -> Vec3 {
    Vec3::new(
        coord.x as f32 * PX_PER_TILE,
        -coord.y as f32 * PX_PER_TILE,
        0.0,
    )
}

fn rotate_coord(coord: IVec2, rotation: Rotation) -> IVec2 {
    let rot2vec = match rotation {
        Rotation::Left => IVec2::new(0, -1),
        Rotation::Right => IVec2::new(0, 1),
    };
    rot2vec.rotate(coord)
}

#[test]
fn test_rotate_coord() {
    let c1 = IVec2::new(0, 0);
    let c2 = IVec2::new(0, -1);
    let c3 = IVec2::new(1, 0);
    let c4 = IVec2::new(1, -1);
    let c5 = IVec2::new(1, 1);

    assert_eq!(rotate_coord(c1, Rotation::Right), c1);
    assert_eq!(rotate_coord(c1, Rotation::Left), c1);
    assert_eq!(rotate_coord(c2, Rotation::Right), c3);
    assert_eq!(rotate_coord(c4, Rotation::Right), c5);
    assert_eq!(rotate_coord(c3, Rotation::Left), c2);
    assert_eq!(rotate_coord(c5, Rotation::Left), c4);
}

//----------------------------------------------------------------------

fn handle_blob_spawned(
    commands: &mut Commands,
    blob: Entity,
    blob_query: &Query<&BlobExtra>,
    block_query: &Query<&BlockExtra>,
    config: &Res<ViewConfig>,
) {
    if let Ok(blobdata) = blob_query.get(blob) {
        assert!(!blobdata.transferred);
        let transform = Transform::from_translation(
            config.factory_topleft + coord_to_translation(blobdata.pivot),
        );
        commands
            .entity(blob)
            .insert_bundle(SpatialBundle::from(transform))
            .insert(BlobRenderState {
                last_pivot: blobdata.pivot,
                rotation_steps: 0,
            })
            .with_children(|cb| {
                // Pivot
                cb.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, crate::Z_OVERLAY),
                    texture: DEFAULT_IMAGE_HANDLE.typed(),
                    ..Default::default()
                })
                .insert(Name::new("Pivot Debug"));
            });
        commands.entity(config.renderer_entity).add_child(blob);

        for &block in blobdata.blocks.iter() {
            let blockdata = block_query.get(block).unwrap();

            // temporary hack until we can expect the game logic to do this for us
            commands.entity(blob).add_child(block);

            commands.entity(block).insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::ONE * PX_PER_TILE),
                    ..Default::default()
                },
                transform: Transform::from_translation(coord_to_translation(blockdata.coordinate)),
                texture: config.brick_image.clone(),
                ..Default::default()
            });
        }
    }
}

fn handle_blob_moved(
    commands: &mut Commands,
    blob: Entity,
    blob_query: &mut Query<(&BlobExtra, &mut BlobRenderState)>,
    config: &Res<ViewConfig>,
) {
    if let Ok((blobdata, mut state)) = blob_query.get_mut(blob) {
        let topleft = if blobdata.transferred {
            config.tetris_topleft
        } else {
            config.factory_topleft
        };
        let start = topleft + coord_to_translation(state.last_pivot);
        let end = topleft + coord_to_translation(blobdata.pivot);

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            TransformPositionLens { start, end },
        );
        commands.entity(blob).insert(Animator::new(tween));
        state.last_pivot = blobdata.pivot;
    }
}

fn handle_blob_rotated(
    commands: &mut Commands,
    blob: Entity,
    rotation: Rotation,
    blob_query: &mut Query<(&BlobExtra, &mut BlobRenderState)>,
    config: &Res<ViewConfig>,
) {
    if let Ok((_, mut state)) = blob_query.get_mut(blob) {
        let start = (state.rotation_steps as f32 * 90.0).to_radians();
        let end = match rotation {
            Rotation::Left => ((state.rotation_steps + 1) as f32 * 90.0).to_radians(),
            Rotation::Right => ((state.rotation_steps - 1) as f32 * 90.0).to_radians(),
        };

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            TransformRotateZLens { start, end },
        );

        commands.entity(blob).insert(Animator::new(tween));

        match rotation {
            Rotation::Left => {
                state.rotation_steps = (state.rotation_steps + 1) % 4;
            }
            Rotation::Right => {
                state.rotation_steps = (state.rotation_steps - 1 + 4) % 4;
            }
        }
    }
}

fn handle_blob_transferred(
    commands: &mut Commands,
    blob: Entity,
    blob_query: &mut Query<(&BlobExtra, &mut BlobRenderState)>,
    config: &Res<ViewConfig>,
) {
    if let Ok((blobdata, mut state)) = blob_query.get_mut(blob) {
        let start = config.factory_topleft + coord_to_translation(state.last_pivot);
        let end = config.tetris_topleft + coord_to_translation(blobdata.pivot);

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            TransformPositionLens { start, end },
        );
        commands.entity(blob).insert(Animator::new(tween));
        state.last_pivot = blobdata.pivot;
    }
}

fn handle_line_remove(
    commands: &mut Commands,
    blocks: &Vec<Entity>,
    block_query: &Query<&BlockExtra>,
    config: &Res<ViewConfig>,
) {
    for &block in blocks.iter() {
        let start = Color::WHITE;
        let end = Color::rgba(1.0, 1.0, 1.0, 0.0);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            SpriteColorLens { start, end },
        );
        commands.entity(block).insert(Animator::new(tween));
    }
}

pub fn handle_view_updates(
    mut commands: Commands,
    mut ev: EventReader<ViewUpdate>,
    blob_query: Query<&BlobExtra>,
    block_query: Query<&BlockExtra>,
    mut rendered_blobs: Query<(&BlobExtra, &mut BlobRenderState)>,
    config: Res<ViewConfig>,
) {
    for ev in ev.iter() {
        match *ev {
            ViewUpdate::BlobSpawned(blob) => {
                handle_blob_spawned(&mut commands, blob, &blob_query, &block_query, &config)
            }
            ViewUpdate::BlobMoved(blob) => {
                handle_blob_moved(&mut commands, blob, &mut rendered_blobs, &config)
            }
            ViewUpdate::BlobRotated(blob, rotation) => {
                handle_blob_rotated(&mut commands, blob, rotation, &mut rendered_blobs, &config)
            }
            ViewUpdate::BlobTransferred(blob) => {
                handle_blob_transferred(&mut commands, blob, &mut rendered_blobs, &config)
            }
            ViewUpdate::LineRemove(ref blocks) => {
                handle_line_remove(&mut commands, blocks, &block_query, &config)
            }
        }
    }
}

//----------------------------------------------------------------------
// Code for demoing the rendering module

fn spawn_demo_blob(commands: &mut Commands) -> Entity {
    let body = crate::bodies::prototype::gen_target_body2();
    let mut blocks = Vec::new();
    for r in 0..Blob::size() {
        for c in 0..Blob::size() {
            if body[Blob::coords_to_idx(r, c)] != 0 {
                blocks.push(
                    commands
                        .spawn()
                        .insert(BlockExtra {
                            coordinate: IVec2::new(c as i32 - 4, r as i32 - 4),
                        })
                        .id(),
                );
            }
        }
    }

    commands
        .spawn()
        .insert(BlobExtra {
            transferred: false,
            pivot: IVec2::default(), //IVec2::new(-1, 4),
            blocks,
        })
        .insert(Name::new("Test Blob"))
        .id()
}

pub fn setup_demo_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut evt: EventWriter<ViewUpdate>,
) {
    let factory_gridsize = IVec2::new(10, 14);
    let factory_pos = Vec3::new(-200.0, 0.0, 0.0);
    let factory = commands
        .spawn_bundle::<SpatialBundle>(Transform::from_translation(factory_pos).into())
        .with_children(|cb| {
            cb.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::new(
                        factory_gridsize.x as f32 * PX_PER_TILE,
                        factory_gridsize.y as f32 * PX_PER_TILE,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .insert(Name::new("Mock Factoryfield"))
        .id();
    let factory_topleft = factory_pos
        + Vec3::new(
            -PX_PER_TILE * (-0.5 + factory_gridsize.x as f32 / 2.0),
            PX_PER_TILE * (-0.5 + factory_gridsize.y as f32 / 2.0),
            0.0,
        );

    let tetris_gridsize = IVec2::new(8, 14);
    let tetris_pos = Vec3::new(400.0, 0.0, 0.0);
    let tetris = commands
        .spawn_bundle::<SpatialBundle>(Transform::from_translation(tetris_pos).into())
        .with_children(|cb| {
            cb.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::new(
                        tetris_gridsize.x as f32 * PX_PER_TILE,
                        tetris_gridsize.y as f32 * PX_PER_TILE,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .insert(Name::new("Mock TetrisField"))
        .id();
    let tetris_topleft = tetris_pos
        + Vec3::new(
            -PX_PER_TILE * (-0.5 + tetris_gridsize.x as f32 / 2.0),
            PX_PER_TILE * (-0.5 + tetris_gridsize.y as f32 / 2.0),
            0.0,
        );

    let renderer_entity = spawn_simple_rendering_entity(&mut commands)
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
        .add_child(factory)
        .add_child(tetris)
        .id();

    let blob = spawn_demo_blob(&mut commands);

    commands.insert_resource(ViewConfig {
        renderer_entity,
        factory_topleft,
        tetris_topleft,
        anim_duration: Duration::from_millis(200),
        brick_image: assets.block_blob.clone(),
        test_blob: Some(blob),
    });

    evt.send(ViewUpdate::BlobSpawned(blob));
}

fn rotate_demo_blob(
    blobdata: &BlobExtra,
    block_query: &mut Query<&mut BlockExtra>,
    rotation: Rotation,
) {
    for &block in blobdata.blocks.iter() {
        let mut blockdata = block_query.get_mut(block).unwrap();
        blockdata.coordinate = rotate_coord(blockdata.coordinate, rotation);
    }
}

fn lowest_blocks_of_testblob(
    block_query: &Query<&mut BlockExtra>,
    blob_query: &Query<&mut BlobExtra>,
    config: &Res<ViewConfig>,
) -> Vec<Entity> {
    let blobdata = blob_query.get(config.test_blob.unwrap()).unwrap();
    let max_y = blobdata
        .blocks
        .iter()
        .map(|&block| {
            let blockdata = block_query.get(block).unwrap();
            blockdata.coordinate.y
        })
        .max()
        .unwrap_or_default();
    blobdata
        .blocks
        .iter()
        .filter_map(|&block| {
            let blockdata = block_query.get(block).unwrap();
            if blockdata.coordinate.y == max_y {
                Some(block)
            } else {
                None
            }
        })
        .collect()
}

pub fn demo_system(
    config: Res<ViewConfig>,
    mut blob_query: Query<&mut BlobExtra>,
    mut block_query: Query<&mut BlockExtra>,
    query: Query<&ActionState<TetrisActionsWASD>>,
    mut evt: EventWriter<ViewUpdate>,
) {
    query.for_each(|s| {
        if s.just_pressed(TetrisActionsWASD::Down) {
            bevy::log::info!("DOWN pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut blobdata) = blob_query.get_mut(test_blob) {
                    blobdata.pivot.y += 1;
                    evt.send(ViewUpdate::BlobMoved(test_blob));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::RRotate) {
            bevy::log::info!("RRotate pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(blobdata) = blob_query.get(test_blob) {
                    rotate_demo_blob(blobdata, &mut block_query, Rotation::Right);
                    evt.send(ViewUpdate::BlobRotated(test_blob, Rotation::Right));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::LRotate) {
            bevy::log::info!("LRotate pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(blobdata) = blob_query.get(test_blob) {
                    rotate_demo_blob(blobdata, &mut block_query, Rotation::Left);
                    evt.send(ViewUpdate::BlobRotated(test_blob, Rotation::Left));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::Up) {
            bevy::log::info!("UP for block transfer pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut blobdata) = blob_query.get_mut(test_blob) {
                    blobdata.transferred = true;
                    blobdata.pivot.y = 0;
                    evt.send(ViewUpdate::BlobTransferred(test_blob));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::Left) {
            bevy::log::info!("LEFT for LineRemove pressed!");
            let to_remove = lowest_blocks_of_testblob(&block_query, &blob_query, &config);
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut blobdata) = blob_query.get_mut(test_blob) {
                    blobdata.blocks.retain(|x| !to_remove.contains(x));
                    evt.send(ViewUpdate::LineRemove(to_remove));
                }
            }
        }
    });
}
