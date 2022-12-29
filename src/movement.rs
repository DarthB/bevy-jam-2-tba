use crate::prelude::*;
use bevy::{log, prelude::*};

pub struct BlobMoveEvent {
    delta: IVec2,

    entity: Entity,
}

pub struct BlobTeleportEvent {
    entity: Entity,
}

pub fn generate_move_events_by_gravity(
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
                    // events are afterwards read in order!
                    ev.send(BlobMoveEvent {
                        delta: IVec2::new(blob.movement.x, blob.movement.y),
                        entity: e,
                    });
                }
            }
        }
    }
}

pub fn move_factory_blobs_by_events(
    mut query: Query<(Entity, &mut Blob, &mut GridBody)>,
    query_tool: Query<&Tool, Without<Blob>>,
    field_query: Query<(Entity, &Field), With<FactoryFieldTag>>,
    mut block_query: Query<(Entity, &mut Block)>,
    mut ev_move: EventReader<BlobMoveEvent>,
    mut ev_teleport: EventWriter<BlobTeleportEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    let (_field_id, field) = field_query.single();

    for ev in ev_move.iter() {
        if let Ok((blob_id, mut blob, mut body)) = query.get_mut(ev.entity) {
            if blob.active && !body.transferred {
                log::info!("Move Factory!");

                let tv = body.pivot + ev.delta;
                let state = field.get_field_state();

                // if blob is at the coordinate limit send an BlobTeleportEvent
                if tv.y >= field.overlap_bottom as i32 {
                    ev_teleport.send(BlobTeleportEvent { entity: ev.entity });
                    continue;
                }
                //~

                // depending on the state at the target position of the grid decide how the movement happens
                // here we just check for the pivot position
                if let Some(element) = state.get_element(tv) {
                    let mut do_move = false;
                    match element.kind {
                        FieldElementKind::Block(by_id) => {
                            do_move = by_id.is_some(); // blobs are allowed to move over each other!
                        }
                        FieldElementKind::Empty => {
                            do_move = true;
                        }
                        FieldElementKind::OutOfMovableRegion => {
                            // only react on outside of x movable region
                            do_move = !(tv.x < 0 || tv.x >= field.mov_size().0 as i32);
                        }
                        FieldElementKind::Tool(tool_entity) => {
                            let tool = query_tool.get(tool_entity).unwrap();
                            do_move = true;
                            match *tool {
                                Tool::Move(d) => {
                                    blob.movement = d.into();
                                }
                                Tool::Rotate(d) => {
                                    log::info!("Rotation tool at {},{}", tv.x, tv.y);
                                    //blob.active = false;
                                    let mut block_iter =
                                        block_query.iter_mut().map(|(_, block)| block);
                                    match d {
                                        RotateDirection::Left => {
                                            body.rotate_left(&mut block_iter, &mut ev_view, blob_id)
                                        }
                                        RotateDirection::Right => body.rotate_right(
                                            &mut block_iter,
                                            &mut ev_view,
                                            blob_id,
                                        ),
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            bevy::log::warn!("Do nothing with target: {tv} but stuck");
                        }
                    }

                    // if flag do_move is set perform the actual move
                    if do_move {
                        let block_iter = block_query
                            .iter_mut()
                            .filter(|(_, block)| block.group == Some(blob_id));

                        move_blob(blob_id, &mut body, ev.delta, block_iter, Some(&mut ev_view));
                    }
                }
            }
        }
    }
}

pub fn move_production_blobs_by_events(
    mut commands: Commands,
    mut query: Query<(Entity, &Blob, &mut GridBody)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut field_query: Query<(Entity, &mut Field), With<ProductionFieldTag>>,
    mut ev: EventReader<BlobMoveEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    if let Ok((_field_id, field)) = field_query.get_single_mut() {
        for ev in ev.iter() {
            if let Ok((blob_id, blob, mut body)) = query.get_mut(ev.entity) {
                if blob.active && body.transferred {
                    log::info!("Move Production!");

                    let occ_coords: Vec<IVec2> = query_block
                        .iter_mut()
                        .filter(|(_, block)| block.group == Some(blob_id))
                        .map(|(_, block)| block.position)
                        .collect();

                    // transform grid
                    let mut occ_coords_delta = occ_coords.clone();
                    for ch in &mut occ_coords_delta {
                        *ch += ev.delta;
                    }

                    let field_state = field.get_field_state();
                    log::info!("Target by move:\n{:?}", occ_coords_delta);
                    let occupied = field_state.is_any_coordinate(
                        &occ_coords_delta,
                        Some(&occ_coords),
                        &|el| {
                            el.kind != FieldElementKind::Empty
                                && el.kind != FieldElementKind::OutOfMovableRegion
                        },
                    );

                    let block_iter = query_block
                        .iter_mut()
                        .filter(|(_, block)| block.group == Some(blob_id));
                    if !occupied {
                        move_blob(blob_id, &mut body, ev.delta, block_iter, Some(&mut ev_view));
                    } else {
                        log::info!("Full Stop and occupy");
                        dissolve_blob(&mut commands, blob_id, block_iter, Some(&mut ev_view));
                    }
                }
            }
        }
    }
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

pub fn teleport_blob<'a>(
    blob_id: Entity,
    body: &mut GridBody,
    field: Entity,
    block_iter: impl Iterator<Item = (Entity, Mut<'a, Block>)>,
    ev_view: Option<&mut EventWriter<ViewUpdate>>,
) {
    // set the transferred flags for the renderer
    body.transferred = true;

    // update the field reference in blocks
    for (_id, mut block) in block_iter {
        if let Some(blob_of_block) = block.group {
            if blob_of_block == blob_id {
                block.field = field;
            }
        }
    }

    // send the event that informs the renderer if event writer is given
    if let Some(ev_view) = ev_view {
        ev_view.send(ViewUpdate::BlobTransferred(blob_id));
    }
}

pub fn handle_teleport_event(
    query_prod: Query<Entity, With<ProductionFieldTag>>,
    mut query_blob: Query<(Entity, &mut GridBody)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut ev: EventReader<BlobTeleportEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    for ev in ev.iter() {
        if let Ok((blob_id, body)) = &mut query_blob.get_mut(ev.entity) {
            if body.transferred {
                log::warn!(
                    "Blob {:?} is already transfered, aborting teleport event.",
                    blob_id
                );
                continue;
            }
            //~

            // collect data
            let prod_field = query_prod.single();
            let target = IVec2::new(body.pivot.x, -3);
            let delta = target - body.pivot;

            // update the grid coordinates
            {
                let block_iter_move = query_block
                    .iter_mut()
                    .filter(|(_, block)| block.group == Some(*blob_id));

                move_blob(*blob_id, body, delta, block_iter_move, None);
            }

            // set the correct transfer flags
            let block_iter_teleport = query_block
                .iter_mut()
                .filter(|(_, block)| block.group == Some(*blob_id));
            teleport_blob(
                *blob_id,
                body,
                prod_field,
                block_iter_teleport,
                Some(&mut ev_view),
            );

            log::info!("Blob teleported!");
        }
    }
}
