use crate::{blob::*, field::spawn_field, prelude::*};
use bevy::{ecs::system::EntityCommands, prelude::*};
use leafwing_input_manager::prelude::*;

use rand::Rng;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameState {
    /// The startup for loading stuff etc.
    Starting,

    /// The ingame state where the actual action happens!
    Ingame,
}

// an example component as plain struct that can be shown in the inspector gui
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
pub struct UpgradeableMover {
    pub speed: f32,

    pub max_speed: f32,

    pub num_powerups: i32,
}

impl UpgradeableMover {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        UpgradeableMover {
            speed: 128.0,
            max_speed: 512.0,
            num_powerups: rng.gen_range(1..4),
        }
    }

    pub fn powerup(&mut self) -> bool {
        if self.num_powerups > 0 {
            self.speed *= 2.0;
            self.speed = self.speed.clamp(128.0, self.max_speed);
            self.num_powerups -= 1;
            bevy::log::info!("Powerup used, {} left.", self.num_powerups);
            true
        } else {
            bevy::log::info!("No more Powerups.");
            false
        }
    }
}

pub fn spawn_world(
    mut commands: Commands, // stores commands for entity/component creation / deletion
    _asset_server: Res<AssetServer>,
    mut turn: ResMut<Turn> // used to access files stored in the assets folder.
) {
    /*
    // wasd bird
    spawn_bird(
        Vec3::new(-200.0, -200.0, 0.0),
        &mut commands, &asset_server,
        &|ec| {
            add_wasd_control(ec);
            ec.insert(Name::new("Bird 1"));
        }
    );


    // arrow bird
    spawn_bird(
        Vec3::new(-200.0, 200.0, 0.0),
        &mut commands, &asset_server,
        &|ec| {
            add_arrow_control(ec);
            ec.insert(Name::new("Bird 2"));
        }
    );
    */

    let comp = Field::as_factory();
    let fac_field = spawn_field(
        &mut commands,
        comp,
        "Factory Field",
        Vec3::new(-350.0, 0.0, 0.0),
        &|ec| {
            ec.insert(FactoryFieldTag {});
        },
    );
    turn.fac_id = Some(fac_field);

    let l_stone = spawn_blob(
        &mut commands,
        bodies::prototype::gen_blob_body(),
        "L Stone",
        Some(Coordinate { c: 3, r: -4 }),
        &|ec| {
            add_tetris_control(ec);
        },
    );
    commands.entity(fac_field).push_children(&[l_stone]);

    let pr_field = spawn_field(
        &mut commands,
        Field::as_production_field(),
        "Production Field",
        Vec3::new(480.0, 0.0, 0.0),
        &|ec| {
            ec.insert(ProductionFieldTag {});
        },
    );
    turn.prod_id = Some(pr_field);

    let t_stone = spawn_blob(
        &mut commands,
        bodies::prototype::gen_target_body(),
        "Target Stone",
        Some(Coordinate { c: 4, r: -3 }),
        &|_| {},
    );
    commands.entity(pr_field).push_children(&[t_stone]);
}

pub fn spawn_bird(
    translation: Vec3,
    commands: &mut Commands,
    asset_server: &AssetServer,
    adapter: &dyn Fn(&mut EntityCommands),
) {
    let mut ec = commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            translation,
            ..Default::default()
        },
        texture: asset_server.load("icon.png"),
        ..Default::default()
    });
    ec.insert(UpgradeableMover::new());
    adapter(&mut ec);
}

pub fn contiously_spawn_tetris_at_end(
    mut commands: Commands,
    query_active: Query<&BlobGravity>,
    turn: ResMut<Turn>,
) {
    if let Some(prod_ent) = turn.prod_id {

        if turn.is_new_turn() && 
            query_active.iter().filter(|g| {!g.is_zero()}).count() == 0 {
            
            let body = gen_random_tetris_body();

            let new_id = spawn_blob(
                &mut commands, 
                body, 
                format!("{}. Additional Tetris Brick", turn.num_additional_bricks).as_str(), 
                Some(Coordinate{r:-3,c:3}), 
                &|ec| {add_tetris_control(ec);}
            );
            commands.entity(prod_ent).push_children(&[new_id]);
        }   
    } else {
        panic!("The programmer forgot to create the production Field...");
    }
}

pub fn apply_powerups_by_actions(
    mut query: Query<(&ActionState<WASDActions>, &mut UpgradeableMover)>,
) {
    for (a, mut um) in query.iter_mut() {
        if a.just_released(WASDActions::Powerup) {
            um.powerup();
        }
    }
}
