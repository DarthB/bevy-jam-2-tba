use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::prelude::*;

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_players_by_actions(
    mut query: Query<(&ActionState<WASDActions>, &UpgradeableMover, &mut Transform)>, // get every entity, that has these three components
    time: Res<Time> // get a bevy-internal resource that represents the time 
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