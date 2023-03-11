use bevy::{ecs::system::EntityCommands, prelude::*};

#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::*;

#[cfg_attr(feature = "debug", derive(InspectorOptions))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Target {
    /// the body of the target, different values of the i32 represent different colors
    pub body: Vec<i32>,

    /// @todo in respect to what?
    pub coordinate: Option<Coordinate>,
}

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
        }
    }

    pub fn coords_to_idx(r: usize, c: usize) -> usize {
        r * Target::dimensions().0 + c
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

pub fn spawn_target(
    commands: &mut Commands,
    body: Vec<i32>,
    name: &str,
    coord: Option<Coordinate>,
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let target = Target {
        body,
        coordinate: coord,
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
