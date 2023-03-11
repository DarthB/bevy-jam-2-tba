use crate::prelude::*;
use bevy::{log, prelude::*};

/// A event that indicates that the Blob shall be moved, can be dispatched by
/// gravity, input and tools. @TODO - Think about an own schedule for the blob movement
/// as a movement of a blob may lead to new movement events that need to be processed
/// immediately (e.g. it collides with a tool)
///
/// If both delta and rot_dir is given then first the rotation is applied and then
/// the blob is moved.
pub struct BlobMoveEvent {
    delta: IVec2,

    entity: Entity,
}

pub fn move_events_by_gravity_system(
    query_collector: Query<(Entity, &GridBody)>,
    mut query: Query<(Entity, &Blob)>,
    turn: Res<Turn>,
    mut ev: EventWriter<BlobMoveEvent>,
) {
    if turn.is_new_turn() {
        let mut vec: Vec<(Entity, i32)> = query_collector
            .iter()
            .map(|(entity, body)| (entity, body.pivot.y))
            .collect();

        vec.sort_by(|left, right| left.1.cmp(&right.1));

        for (id, _) in vec {
            if let Ok((e, blob)) = query.get_mut(id) {
                if blob.active {
                    ev.send(BlobMoveEvent {
                        delta: IVec2::new(blob.movement.x, blob.movement.y),
                        entity: e,
                    });
                }
            }
        }
    }
}

pub fn handle_move_blob_events(
    mut commands: Commands,
    mut query: Query<(Entity, &Blob, &mut GridBody)>,
    field_query: Query<(Entity, &Field)>,
    mut block_query: Query<(Entity, &mut Block)>,
    mut ev_move: EventReader<BlobMoveEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    let (_field_id, field) = if let Ok(pair) = field_query.get_single() {
        pair
    } else {
        return;
    };
    //~

    for ev in ev_move.iter() {
        if let Ok((blob_id, blob, mut body)) = query.get_mut(ev.entity) {
            if blob.active && !body.transferred {
                log::info!("Move Factory!");

                // get block positions of the blob
                let occ_coords: Vec<IVec2> = block_query
                    .iter_mut()
                    .filter(|(_, block)| block.group == Some(blob_id))
                    .map(|(_, block)| block.position)
                    .collect();

                // transform blocks with given delta
                let mut occ_coords_delta = occ_coords.clone();
                for ch in &mut occ_coords_delta {
                    *ch += ev.delta;
                }

                // check if any coordinate of the movement target is already occupied, e.g by a previous blob
                let state = field.get_field_state();
                let mut do_move =
                    !state.is_any_coordinate(&occ_coords_delta, Some(&occ_coords), &|el| match el
                        .kind
                    {
                        FieldElementKind::Empty
                        | FieldElementKind::OutOfMovableRegion
                        | FieldElementKind::Tool(_) => false,
                        FieldElementKind::Block(id) if id.is_some() => false,
                        _ => true,
                    }) || blob.cutout;

                // hack: we don't want the cutout blobs to interfer with the playfield therefore we move them away
                let delta = if blob.cutout {
                    IVec2::new(-2, 3 * ev.delta.y)
                } else {
                    ev.delta
                };

                // depending on the state at the target position of the grid decide how the movement happens
                // here we just check for the pivot position
                if !blob.cutout || do_move {
                    do_move = handle_move(&mut body, delta, field, &mut block_query);
                }

                // if flag do_move is set perform the actual move
                let block_iter = block_query
                    .iter_mut()
                    .filter(|(_, block)| block.group == Some(blob_id));
                if do_move {
                    move_blob(blob_id, &mut body, delta, block_iter, Some(&mut ev_view));
                } else {
                    log::info!("Full Stop and occupy");
                    dissolve_blob(&mut commands, blob_id, block_iter, Some(&mut ev_view));
                }
            }
        }
    }
}

fn handle_move(
    body: &mut GridBody,
    delta: IVec2,
    field: &Field,
    block_query: &mut Query<(Entity, &mut Block)>,
) -> bool {
    let mut do_move = false;

    let state = field.get_field_state();
    let rel_pos = body.get_relative_positions(block_query);
    let do_move = rel_pos.iter().all(|pos| {
        let ap = *pos + body.pivot + delta;

        if let Some(element) = state.get_element(ap) {
            match element.kind {
                FieldElementKind::Block(by_id) => {
                    do_move = by_id.is_some(); // blobs are allowed to move over each other!
                }
                FieldElementKind::Empty => {
                    do_move = true;
                }
                FieldElementKind::OutOfMovableRegion => {
                    // only react on outside of x movable region
                    log::info!("{} < {}?", ap.y, field.movable_size.1);
                    do_move = ap.y < field.movable_size.1 as i32;
                }
                FieldElementKind::Tool(_) => {
                    do_move = true;
                }
                _ => {
                    bevy::log::warn!("Do nothing with target: {} but stuck", ap);
                }
            }
        } else {
            // we allow to leave the field on the top
            do_move = ap.y < 0;
        }

        do_move
    });

    do_move
}

pub fn dissolve_blob<'a>(
    commands: &mut Commands,
    blob_id: Entity,
    block_iter: impl Iterator<Item = (Entity, Mut<'a, Block>)>,
    _ev_view: Option<&mut EventWriter<ViewUpdate>>,
) {
    commands.entity(blob_id).despawn();

    for (_, mut block) in block_iter {
        block.group = None;
    }
}

pub fn move_blob<'a>(
    blob_id: Entity,
    body: &mut GridBody,
    delta: IVec2,
    block_iter: impl Iterator<Item = (Entity, Mut<'a, Block>)>,
    ev_view: Option<&mut EventWriter<ViewUpdate>>,
) {
    // update the grid position
    body.pivot += delta;

    // update the grid position of every block
    for (_id, mut block) in block_iter {
        if let Some(blob_of_block) = block.group {
            if blob_of_block == blob_id {
                block.position += delta;
            }
        }
    }

    // send event to renderer if event writer is present
    if let Some(ev_view) = ev_view {
        ev_view.send(ViewUpdate::BlobMoved(blob_id));
    }
}
