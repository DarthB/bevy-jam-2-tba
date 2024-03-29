//! # Module for presentation and animations
//!
//!
//!

use std::{
    collections::HashMap,
    ops::{Add, Mul},
    time::Duration,
};

use crate::{
    data::assets::GameAssets, field::prelude::*, input::TetrisActionsWASD, DisastrisAppState,
    PX_PER_TILE, Z_SOLID,
};
use bevy::{ecs::system::EntityCommands, log, prelude::*};
use bevy_tweening::{lens::*, *};
use interpolation::{Ease, EaseFunction};
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

//----------------------------------------------------------------------

pub mod prelude {
    pub use super::Rotation;
    pub use super::ViewConfig;
    pub use super::ViewUpdate;

    pub use super::rotate_coord;
}

//----------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum Rotation {
    Left,
    Right,
}

/// Events for the Renderer
#[derive(Event)]
pub enum ViewUpdate {
    /// A new blob has been spawned in the factory
    BlobSpawned(Entity),
    /// A blob has been moved
    BlobMoved(Entity),
    /// A blob has been rotated
    BlobRotated(Entity, Rotation),
    /// A new blob `entity` has been cutout. The new blob `entity` must have the
    /// blocks that originally were part of the original blob.
    BlobCutout(Entity),
    /// A blob has been transferred from the factory to tetris arena
    BlobTransferred(Entity),
    /// A line of blocks was removed in the tetris field.
    LineRemove(Vec<Entity>),
}

/// Configuration struct for integration into the rest of the project
#[derive(Resource)]
pub struct ViewConfig {
    /// Global renderer entity just for making the entity tree a bit cleaner.
    /// Create a basic one via `spawn_simple_rendering_entity`.
    pub renderer_entity: Entity,
    /// Global translation to the factory fields's (0,0) block
    pub factory_topleft: Vec3,
    /// Global translation to the tetris fields's (0,0) block
    pub tetris_topleft: Vec3,
    /// Animation duration
    pub anim_duration: Duration,

    pub brick_image: Handle<Image>,

    /// Used by the demo system (can be ignored)
    pub test_blob: Option<Entity>,
}

pub fn spawn_simple_rendering_entity<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
) -> EntityCommands<'w, 's, 'a> {
    let mut rv = commands.spawn(SpatialBundle::default());
    rv.insert(Name::new("Rendering"));
    rv
}

//----------------------------------------------------------------------
// Internal helper components for rendering

#[derive(Component, Clone)]
pub struct BlobRenderState {
    /// Last known `pivot` from the game logic
    last_pivot: IVec2,
    /// Cumulative left rotations (0..3)
    rotation_steps: i32,

    //---------------------
    rotation_tween: MyTween<f32>,
    translation_tween: MyTween<Vec3>,
}

//----------------------------------------------------------------------
// Tweening

/// Simple tweening over a vector space T.
#[derive(Clone)]
struct MyTween<T> {
    start: T,
    end: T,
    ease: EaseFunction,
    elapsed: Duration,
    duration: Duration,
}

impl<T> MyTween<T>
where
    T: Copy + Add<T, Output = T> + Mul<f32, Output = T> + Default,
{
    pub fn new(end: T) -> Self {
        Self {
            start: Default::default(),
            end,
            ease: EaseFunction::QuadraticOut,
            elapsed: Duration::ZERO,
            duration: Duration::ZERO,
        }
    }

    pub fn tick(&mut self, elapsed: Duration) -> T {
        self.elapsed += elapsed;
        if self.elapsed >= self.duration {
            self.end
        } else {
            let factor = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
            debug_assert!((0.0..=1.0).contains(&factor));
            let factor = factor.calc(self.ease);
            self.start * (1.0 - factor) + self.end * factor
        }
    }

    pub fn set(&mut self, start: T, end: T, duration: Duration) {
        *self = Self {
            start,
            end,
            duration,
            elapsed: Duration::ZERO,
            ..*self
        };
    }
}

//----------------------------------------------------------------------
// utility functions

fn coord_to_translation(coord: IVec2) -> Vec3 {
    Vec3::new(
        coord.x as f32 * PX_PER_TILE,
        -coord.y as f32 * PX_PER_TILE,
        Z_SOLID,
    )
}

pub fn rotate_coord(coord: IVec2, rotation: Rotation) -> IVec2 {
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
// animation system

pub fn animate_rendered_blob_system(
    mut rendered_blobs: Query<(&mut BlobRenderState, &mut Transform)>,
    time: Res<Time>,
) {
    let elapsed = time.delta();
    for (mut state, mut transform) in rendered_blobs.iter_mut() {
        let new_rotation = state.rotation_tween.tick(elapsed);
        transform.rotation = Quat::from_rotation_z(new_rotation);
        transform.translation = state.translation_tween.tick(elapsed);
    }
}

//----------------------------------------------------------------------
// handle ViewUpdate events

fn handle_blob_spawned(
    commands: &mut Commands,
    blob: Entity,
    blob_query: &Query<&GridBody>,
    block_query: &Query<&Block>,
    config: &Res<ViewConfig>,
) {
    log::info!("Handle spawned: {:?}!", blob);
    let bodydata = blob_query.get(blob).unwrap();
    assert!(!bodydata.transferred);
    let transform =
        Transform::from_translation(config.factory_topleft + coord_to_translation(bodydata.pivot));
    commands
        .entity(blob)
        .insert(SpatialBundle::from(transform))
        .insert(BlobRenderState {
            last_pivot: bodydata.pivot,
            rotation_steps: 0,
            rotation_tween: MyTween::new(0.0),
            translation_tween: MyTween::new(transform.translation),
        })
        .with_children(|cb| {
            // Pivot
            cb.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                    ..Default::default()
                },
                transform: Transform::from_xyz(0.0, 0.0, crate::Z_OVERLAY),
                ..Default::default()
            })
            .insert(Name::new("Pivot Debug"));
        });
    commands.entity(config.renderer_entity).add_child(blob);

    for &block in bodydata.blocks.iter() {
        let blockdata = block_query.get(block).unwrap();

        // temporary hack until we can expect the game logic to do this for us
        commands.entity(blob).add_child(block);

        commands.entity(block).insert(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::ONE * PX_PER_TILE),
                ..Default::default()
            },
            transform: Transform::from_translation(coord_to_translation(
                blockdata.relative_position.unwrap(),
            )),
            texture: config.brick_image.clone(),
            ..Default::default()
        });
    }
}

fn handle_blob_moved(
    _commands: &mut Commands,
    blob: Entity,
    blob_query: &mut Query<(&GridBody, &mut BlobRenderState)>,
    config: &Res<ViewConfig>,
) {
    if let Ok((bodydata, mut state)) = blob_query.get_mut(blob) {
        let topleft = if bodydata.transferred {
            config.tetris_topleft
        } else {
            config.factory_topleft
        };
        let start = topleft + coord_to_translation(state.last_pivot);
        let end = topleft + coord_to_translation(bodydata.pivot);

        /*
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            TransformPositionLens { start, end },
        );
        commands.entity(blob).insert(Animator::new(tween));*/
        state
            .translation_tween
            .set(start, end, config.anim_duration);

        state.last_pivot = bodydata.pivot;
    }
}

fn handle_blob_rotated(
    _commands: &mut Commands,
    blob: Entity,
    rotation: Rotation,
    blob_query: &mut Query<(&GridBody, &mut BlobRenderState)>,
    config: &Res<ViewConfig>,
) {
    if let Ok((_, mut state)) = blob_query.get_mut(blob) {
        let start = (state.rotation_steps as f32 * 90.0).to_radians();
        let end = match rotation {
            Rotation::Left => ((state.rotation_steps + 1) as f32 * 90.0).to_radians(),
            Rotation::Right => ((state.rotation_steps - 1) as f32 * 90.0).to_radians(),
        };

        /*
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            TransformRotateZLens { start, end },
        );
        commands.entity(blob).insert(Animator::new(tween));
        */
        state.rotation_tween.set(start, end, config.anim_duration);

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

fn handle_blob_cutout(
    commands: &mut Commands,
    newblob: Entity,
    blob_query: &Query<&GridBody>,
    block_query: &Query<&Block>,
    config: &Res<ViewConfig>,
) {
    handle_blob_spawned(commands, newblob, blob_query, block_query, config);
    let body = blob_query.get(newblob).unwrap();
    for &block in body.blocks.iter() {
        let tween = Tween::new(
            EaseFunction::BounceInOut,
            config.anim_duration,
            SpriteColorLens {
                start: Color::WHITE,
                end: Color::BLUE,
            },
        )
        .then(Tween::new(
            EaseFunction::BounceInOut,
            config.anim_duration,
            SpriteColorLens {
                start: Color::BLUE,
                end: Color::GRAY,
            },
        ));
        commands.entity(block).insert(Animator::new(tween));
    }
}

fn handle_blob_transferred(
    _commands: &mut Commands,
    blob: Entity,
    blob_query: &mut Query<(&GridBody, &mut BlobRenderState)>,
    config: &Res<ViewConfig>,
) {
    if let Ok((blobdata, mut state)) = blob_query.get_mut(blob) {
        let start = config.factory_topleft + coord_to_translation(state.last_pivot);
        let end = config.tetris_topleft + coord_to_translation(blobdata.pivot);

        /*
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            config.anim_duration,
            TransformPositionLens { start, end },
        );
        commands.entity(blob).insert(Animator::new(tween));
        */
        state
            .translation_tween
            .set(start, end, config.anim_duration);

        state.last_pivot = blobdata.pivot;
    }
}

fn handle_line_remove(
    commands: &mut Commands,
    blocks: &[Entity],
    _block_query: &Query<&Block>,
    config: &Res<ViewConfig>,
) {
    for &block in blocks.iter() {
        let start = Color::WHITE;
        let end = Color::rgba(1.0, 1.0, 1.0, 0.0);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            config.anim_duration,
            SpriteColorLens { start, end },
        );
        commands.entity(block).insert(Animator::new(tween));
    }
}

pub fn handle_view_update_system(
    mut commands: Commands,
    mut ev: EventReader<ViewUpdate>,
    blob_query: Query<&GridBody>,
    block_query: Query<&Block>,
    mut rendered_blobs: Query<(&GridBody, &mut BlobRenderState)>,
    config: Res<ViewConfig>,
) {
    for ev in ev.read() {
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
            ViewUpdate::BlobCutout(newblob) => {
                handle_blob_cutout(&mut commands, newblob, &blob_query, &block_query, &config)
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

/// Registers the animation demo
/// strongly simplified as we also rely on global app setup in [`crate::start_disastris`]
pub fn register_animation_demo(app: &mut App) {
    app.add_systems(OnEnter(DisastrisAppState::AnimationTest), setup_demo_system);
    app.add_systems(Update, demo_system);
}

fn spawn_demo_blob(commands: &mut Commands) -> Entity {
    let field_id = commands.spawn_empty().id();
    let body = crate::data::bodies::gen_blob_body(1).unwrap();
    let mut rel_pos = vec![];
    let mut blocks = Vec::new();
    for r in 0..GridBody::size() {
        for c in 0..GridBody::size() {
            if body[GridBody::coords_to_idx(r, c)] != 0 {
                let coord = IVec2::new(c as i32 - 4, r as i32 - 4);
                rel_pos.push(coord);
                blocks.push(
                    commands
                        .spawn_empty()
                        .insert(Block {
                            relative_position: Some(coord),
                            position: coord,
                            group: None,
                            field: field_id,
                        })
                        .id(),
                );
            }
        }
    }

    commands
        .spawn_empty()
        .insert(GridBody {
            pivot: IVec2::default(),
            blocks,
            transferred: false,
        })
        .insert(Blob {
            movement: IVec2::ZERO, //IVec2::new(-1, 4),
            active: true,
            cutout: false,
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
        .spawn::<SpatialBundle>(Transform::from_translation(factory_pos).into())
        .with_children(|cb| {
            cb.spawn(SpriteBundle {
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
        .spawn::<SpatialBundle>(Transform::from_translation(tetris_pos).into())
        .with_children(|cb| {
            cb.spawn(SpriteBundle {
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
        .insert(InputManagerBundle::<TetrisActionsWASD> {
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

fn rotate_demo_blob(blobdata: &GridBody, block_query: &mut Query<&mut Block>, rotation: Rotation) {
    for &block in blobdata.blocks.iter() {
        let mut blockdata = block_query.get_mut(block).unwrap();
        blockdata.relative_position =
            Some(rotate_coord(blockdata.relative_position.unwrap(), rotation));
    }
}

fn lowest_blocks_of_testblob(
    block_query: &Query<&mut Block>,
    blob_query: &Query<&mut GridBody>,
    config: &Res<ViewConfig>,
) -> Vec<Entity> {
    let blobdata = blob_query.get(config.test_blob.unwrap()).unwrap();
    let max_y = blobdata
        .blocks
        .iter()
        .map(|&block| {
            let blockdata = block_query.get(block).unwrap();
            blockdata.relative_position.unwrap().y
        })
        .max()
        .unwrap_or_default();
    blobdata
        .blocks
        .iter()
        .filter_map(|&block| {
            let blockdata = block_query.get(block).unwrap();
            if blockdata.relative_position.unwrap().y == max_y {
                Some(block)
            } else {
                None
            }
        })
        .collect()
}

/// Tries to cutout a triangle from `blob` by
/// - finding 3 block entities from the test blob forming a triangle
/// - spawn a new blob entity and reattaches above blocks to the new blob entity
///   and set the new blob's pivot coordinate accordingly
/// - returns the new blob entity.
fn cutout_triangle_from_blob(
    commands: &mut Commands,
    blob: Entity,
    blob_query: &mut Query<&mut GridBody>,
    block_query: &mut Query<&mut Block>,
) -> Option<Entity> {
    let mut blobdata = blob_query.get_mut(blob).unwrap();
    let coordinates = blobdata
        .blocks
        .iter()
        .map(|&block| {
            let blockdata = block_query.get(block).unwrap();
            (blockdata.relative_position.unwrap(), block)
        })
        .collect::<HashMap<IVec2, Entity>>();
    let triangle_blocks = coordinates.iter().find_map(|(coord, &block)| {
        let a = coordinates.get(&IVec2::new(coord.x + 1, coord.y));
        let b = coordinates.get(&IVec2::new(coord.x, coord.y + 1));
        match (a, b) {
            (Some(&a), Some(&b)) => Some((block, a, b)),
            _ => None,
        }
    });
    if let Some((a, b, c)) = triangle_blocks {
        let pivot_block_coordinate = block_query.get(a).unwrap().relative_position.unwrap();
        let pivot = pivot_block_coordinate + blobdata.pivot;
        let blocks = vec![a, b, c];
        blobdata.blocks.retain(|x| !blocks.contains(x));
        for &block in blocks.iter() {
            *block_query
                .get_mut(block)
                .unwrap()
                .relative_position
                .as_mut()
                .unwrap() -= pivot_block_coordinate;
        }
        let newblob = commands
            .spawn_empty()
            .insert_children(0, &blocks[..])
            .insert(GridBody {
                pivot,
                blocks,
                transferred: false,
            })
            .insert(Blob {
                movement: IVec2::ZERO,
                active: true,
                cutout: true,
            })
            .id();
        Some(newblob)
    } else {
        None
    }
}

pub fn demo_system(
    mut commands: Commands,
    config: Res<ViewConfig>,
    mut body_query: Query<&mut GridBody>,
    mut block_query: Query<&mut Block>,
    query: Query<&ActionState<TetrisActionsWASD>>,
    mut evt: EventWriter<ViewUpdate>,
) {
    query.for_each(|s| {
        if s.just_pressed(TetrisActionsWASD::Down) {
            bevy::log::info!("DOWN pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut bodydata) = body_query.get_mut(test_blob) {
                    bodydata.pivot.y += 1;
                    evt.send(ViewUpdate::BlobMoved(test_blob));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::RRotate) {
            bevy::log::info!("RRotate pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(bodydata) = body_query.get(test_blob) {
                    rotate_demo_blob(bodydata, &mut block_query, Rotation::Right);
                    evt.send(ViewUpdate::BlobRotated(test_blob, Rotation::Right));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::LRotate) {
            bevy::log::info!("LRotate pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(bodydata) = body_query.get(test_blob) {
                    rotate_demo_blob(bodydata, &mut block_query, Rotation::Left);
                    evt.send(ViewUpdate::BlobRotated(test_blob, Rotation::Left));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::Up) {
            bevy::log::info!("UP for block transfer pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut bodydata) = body_query.get_mut(test_blob) {
                    bodydata.transferred = true;
                    bodydata.pivot.y = 0;
                    evt.send(ViewUpdate::BlobTransferred(test_blob));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::Left) {
            bevy::log::info!("LEFT for LineRemove pressed!");
            let to_remove = lowest_blocks_of_testblob(&block_query, &body_query, &config);
            if let Some(test_blob) = config.test_blob {
                if let Ok(mut bodydata) = body_query.get_mut(test_blob) {
                    bodydata.blocks.retain(|x| !to_remove.contains(x));
                    evt.send(ViewUpdate::LineRemove(to_remove));
                }
            }
        }
        if s.just_pressed(TetrisActionsWASD::Right) {
            bevy::log::info!("RIGHT for Cutout pressed!");
            if let Some(test_blob) = config.test_blob {
                if let Some(newblob) = cutout_triangle_from_blob(
                    &mut commands,
                    test_blob,
                    &mut body_query,
                    &mut block_query,
                ) {
                    evt.send(ViewUpdate::BlobCutout(newblob));
                }
            }
        }
    });
}
