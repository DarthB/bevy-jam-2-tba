use crate::prelude::*;
use bevy::{log, prelude::*};

pub struct BlobMoveEvent {
    delta: (i32, i32),

    entity: Entity,
}

pub struct BlobTeleportEvent {
    entity: Entity,
}

pub fn generate_move_events_by_gravity(
    query_collector: Query<(Entity, &Blob)>,
    mut query: Query<(Entity, &Blob)>,
    turn: Res<Turn>,
    mut ev: EventWriter<BlobMoveEvent>,
) {
    if turn.is_new_turn() {
        let mut vec: Vec<(Entity, i32)> = query_collector
            .iter()
            .map(|(entity, blob)| (entity, blob.coordinate.y))
            .collect();

        vec.sort_by(|left, right| left.1.cmp(&right.1));

        for (id, _) in vec {
            if let Ok((e, blob)) = query.get_mut(id) {
                if blob.active {
                    // events are afterwards read in order!
                    ev.send(BlobMoveEvent {
                        delta: (blob.movement.x, blob.movement.y),
                        entity: e,
                    });
                }
            }
        }
    }
}

pub fn move_factory_blobs_by_events(
    mut query: Query<(&Parent, Entity, &mut Blob)>,
    parent_query: Query<(Entity, &Field), With<FactoryFieldTag>>,
    mut block_query: Query<&mut Block>,
    mut ev_move: EventReader<BlobMoveEvent>,
    mut ev_teleport: EventWriter<BlobTeleportEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    for ev in ev_move.iter() {
        if let Ok((p, blob_id, mut blob)) = query.get_mut(ev.entity) {
            let (entity, field) = parent_query.single();
            if entity != p.get() {
                continue;
            }
            //~

            if blob.active {
                let (tc, tr) = (blob.coordinate.x + ev.delta.0, blob.coordinate.y + ev.delta.1);
                let tv = IVec2::new(tc, tr);
                let state = field.get_field_state();

                if tr >= field.coordinate_limits().bottom {
                    ev_teleport.send(BlobTeleportEvent { entity: ev.entity });
                    continue;
                }
                //~

                if let Some(element) = state.get_element(tv) {
                    match element.kind {
                        FieldElementKind::Empty => {
                            blob.coordinate = IVec2::new(tc, tr);
                            ev_view.send(ViewUpdate::BlobMoved(blob_id));
                        },
                        FieldElementKind::Tool(tool) => {
                            let mut handled_by_tool = true;
                        
                            match tool {
                                Tool::Move(d) => {
                                    blob.movement = d.into();
                                },
                                Tool::Rotate(d) => {
                                    let block_iter = block_query.iter_mut()
                                        .filter(|block| block.blob.is_some() && block.blob.unwrap() == blob_id );
                                    match d {
                                        RotateDirection::Left => blob.rotate_left(block_iter),
                                        RotateDirection::Right => blob.rotate_right(block_iter),
                                    }
                                },
                                Tool::Cutter(_) => {
                                    // @todo implement cutter tool
                                }
                                _ => {handled_by_tool = false},
                            }

                            if handled_by_tool {
                                blob.coordinate = IVec2::new(tc, tr);
                                ev_view.send(ViewUpdate::BlobMoved(blob_id));
                            }
                        },
                        _ => {
                            bevy::log::warn!("Do nothing with target: {tc}, {tr} but stuck");
                        }
                    }
                }
            }
        }
    }
}

pub fn move_production_blobs_by_events(
    mut commands: Commands,
    mut query: Query<(Entity, &Parent, &mut Blob)>,
    mut parent_query: Query<&mut Field, With<ProductionFieldTag>>,
    mut ev: EventReader<BlobMoveEvent>,
) {
    for ev in ev.iter() {
        if let Ok((id, p, mut blob)) = query.get_mut(ev.entity) {
            
            if let Ok(field) = parent_query.get_mut(p.get()) {
                let occ_coords: Vec<(i32, i32)> = blob
                    .occupied_coordinates()
                    .iter()
                    .map(|(x, y)| (x - pivot_coord().0 as i32, y - pivot_coord().1 as i32))
                    .collect();

                // transform grid
                let mut occ_coords_delta = occ_coords.clone();
                for ch in &mut occ_coords_delta {
                    ch.0 += ev.delta.0;
                    ch.1 += ev.delta.1;
                }

                // test for occupied
                let occ_coords_delta_ivec = occ_coords_delta.iter()
                    .map(|(x, y)| IVec2::new(*x, *y))
                    .collect();
                let field_state = field.get_field_state();
                let occupied = field_state.is_any_coordinate_occupied(&occ_coords_delta_ivec);

                if !occupied {
                    blob.coordinate.x += ev.delta.0;
                    blob.coordinate.y += ev.delta.1;
                } else {
                    log::info!("Full Stop and occupy");
                    blob.active = false;
                    //field.occupy_coordinates(&occ_coords);

                    commands.entity(id).despawn_recursive();
                }
            }
        }
    }
}

pub fn teleport_blob_out_of_factory(
    mut commands: Commands,
    query_fac: Query<Entity, With<FactoryFieldTag>>,
    query_prod: Query<Entity, With<ProductionFieldTag>>,
    mut query_blob: Query<&mut Blob>,
    mut ev: EventReader<BlobTeleportEvent>,
) {
    for ev in ev.iter() {
        if let Ok(blob) = &mut query_blob.get_mut(ev.entity) {
            let fac = query_fac.single();
            let prod = query_prod.single();

            commands.entity(fac).remove_children(&[ev.entity]);
            blob.coordinate.y = -3;
            commands.entity(prod).push_children(&[ev.entity]);
        }
    }
}
