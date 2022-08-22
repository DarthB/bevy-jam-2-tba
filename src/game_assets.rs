use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct GameAssets {
    pub blob_image: Handle<Image>,

    pub direction_d: Handle<Image>,

    pub direction_l: Handle<Image>,

    pub direction_r: Handle<Image>,

    pub direction_u: Handle<Image>,

    pub factory_floor: Handle<Image>,

    pub rotate_l: Handle<Image>,

    pub rotate_r: Handle<Image>,

    pub target_outline: Handle<Image>,

    pub tetris_floor: Handle<Image>,
}
