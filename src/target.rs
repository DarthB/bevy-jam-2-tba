use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Target {
    /// the body of the target, different values of the i32 represent different colors
    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub body: Vec<i32>,

    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub texture: Handle<Image>,

    /// @todo in respect to what?
    pub coordinate: Option<Coordinate>,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub struct Coordinate {
    pub r: i32,
    pub c: i32,
}

impl From<Coordinate> for (i32, i32) {
    fn from(c: Coordinate) -> Self {
        (c.c, c.r)
    }
}

impl From<(i32, i32)> for Coordinate {
    fn from((c, r): (i32, i32)) -> Self {
        Coordinate { r, c }
    }
}

impl Target {
    pub fn new(body: Vec<i32>) -> Self {
        Target {
            body,
            coordinate: None,
            texture: Handle::default(),
        }
    }

    pub fn coords_to_idx(r: usize, c: usize) -> usize {
        coords_to_idx(r, c, Target::dimensions().0)
    }

    pub fn dimensions() -> (usize, usize) {
        (10, 12)
    }

    /// the function calculates the occupied coordinates in the coordinate system of the
    /// parent (coordinate property)
    pub fn occupied_coordinates(&self) -> Vec<(i32, i32)> {
        let mut reval = Vec::new();
        if let Some(coordinate) = self.coordinate {
            for r in 0..Target::dimensions().1 {
                for c in 0..Target::dimensions().0 {
                    if self.body[Target::coords_to_idx(r, c)] != 0 {
                        let c = c as i32 + coordinate.c;
                        let r = r as i32 + coordinate.r;
                        reval.push((c, r));
                    }
                }
            }
        }
        reval
    }
}

pub fn coords_to_idx(r: usize, c: usize, cs: usize) -> usize {
    r * cs + c
}

pub fn coords_to_px(x: i32, y: i32, rs: usize, cs: usize) -> (f32, f32) {
    (
        ((cs as f32 / -2.0) + x as f32) * PX_PER_TILE + PX_PER_TILE / 2.0,
        ((rs as f32 / 2.0) - y as f32) * PX_PER_TILE - PX_PER_TILE / 2.0,
    )
}

pub fn spawn_target(
    commands: &mut Commands,
    texture: &Handle<Image>,
    body: Vec<i32>,
    name: &str,
    coord: Option<Coordinate>,
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let target = Target {
        body,
        coordinate: coord,
        texture: texture.clone(),
    };

    let mut ec = commands.spawn(SpatialBundle {
        transform: Transform::from_translation(Vec3::new(344.0, -192.0, 0.0)),
        ..Default::default()
    });
    let id = ec.id();
    ec.insert(target).insert(Name::new(name.to_string()));
    adapter(&mut ec);

    id
}
