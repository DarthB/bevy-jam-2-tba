use crate::{blob::BlobGravity, prelude::*};
use bevy::{log, prelude::*};
use leafwing_input_manager::prelude::*;

pub struct BlobMoveEvent {
    delta: (i32, i32),

    entity: Entity,
}

pub struct BlobTeleportEvent {
    entity: Entity,
}

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_players_by_actions(
    mut query: Query<(&ActionState<WASDActions>, &UpgradeableMover, &mut Transform)>, // get every entity, that has these three components
    time: Res<Time>, // get a bevy-internal resource that represents the time
) {
    query.for_each_mut(|(s, um, mut t)| {
        let mut dir = Vec2::ZERO;

        if s.pressed(WASDActions::Up) {
            dir.y += 1.0;
        }

        if s.pressed(WASDActions::Down) {
            dir.y -= 1.0;
        }

        if s.pressed(WASDActions::Left) {
            dir.x -= 1.0;
        }

        if s.pressed(WASDActions::Right) {
            dir.x += 1.0;
        }

        let move_dt = dir * time.delta_seconds() * um.speed;
        t.translation += Vec3::new(move_dt.x, move_dt.y, 0.0);
    });
}

pub fn move_blobs_by_gravity(
    mut query: Query<(Entity, &mut Transform, &BlobGravity, &Blob)>,
    turn: Res<Turn>,
    mut ev: EventWriter<BlobMoveEvent>,
) {
    if turn.is_new_turn() {
        query.for_each_mut(|(e, mut t, g, blob)| {
            if blob.coordinate.is_some() {
                ev.send(BlobMoveEvent {
                    delta: g.gravity,
                    entity: e,
                });
            } else {
                // fallback if not ona field
                t.translation.x -= g.gravity.0 as f32 * PX_PER_TILE;
                t.translation.y -= g.gravity.1 as f32 * PX_PER_TILE;
            }
        });
    }
}

pub fn move_factory_blobs_by_events(
    mut query: Query<(&Parent, &mut Blob)>,
    parent_query: Query<&Field, With<FactoryFieldTag>>,
    mut ev: EventReader<BlobMoveEvent>,
    mut ev_teleport: EventWriter<BlobTeleportEvent>,
) {
    for ev in ev.iter() {
        if let Ok((p, mut blob)) = query.get_mut(ev.entity) {
            if let Ok(field) = parent_query.get(p.get()) {
                if let Some(coord) = &mut blob.coordinate {
                    let (tc, tr) = (coord.c + ev.delta.0, coord.r + ev.delta.1);

                    let (occupied, _) = field.is_coordinate_occupied(tc, tr);
                    if !occupied {
                        coord.c = tc;
                        coord.r = tr;
                    } else if tr >= field.coordinate_limits().bottom {
                        ev_teleport.send(BlobTeleportEvent { entity: ev.entity });
                    } else {
                        bevy::log::warn!("Do nothing with target: {tc}, {tr}");
                    }
                }
            }
        }
    }
}

pub fn move_production_blobs_by_events(
    mut query: Query<(&Parent, &mut Blob, &mut BlobGravity)>,
    mut parent_query: Query<&mut Field, With<ProductionFieldTag>>,
    mut ev: EventReader<BlobMoveEvent>,
) {
    for ev in ev.iter() {
        if let Ok((p, mut blob, mut grav)) = query.get_mut(ev.entity) {
            if let Ok(mut field) = parent_query.get_mut(p.get()) {
                let mut occ_coords = blob.occupied_coordinates();

                if let Some(coord) = &mut blob.coordinate {
                    let (tc, tr) = (coord.c + ev.delta.0, coord.r + ev.delta.1);

                    // transform grid
                    let mut occ_coords_test = occ_coords.clone();
                    for ch in &mut occ_coords_test {
                        ch.0 += tc;
                        ch.1 += tr;
                    }

                    // test for occupied
                    let (occupied, _) = field.any_coordinate_occupied(&occ_coords_test);

                    if !occupied {
                        coord.c = tc;
                        coord.r = tr;
                    } else {
                        grav.gravity = (0, 0);
                        for ch in &mut occ_coords {
                            ch.0 += coord.c;
                            ch.1 += coord.r;
                        }
                        log::info!("Full Stop and occupy");
                        field.occupy_coordinates(&occ_coords);
                        // @todo play plong sound or similar
                    }
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
            if let Some(coords) = &mut blob.coordinate {
                let fac = query_fac.single();
                let prod = query_prod.single();

                commands.entity(fac).remove_children(&[ev.entity]);
                coords.r = -3;
                commands.entity(prod).push_children(&[ev.entity]);
            }
        }
    }
}
