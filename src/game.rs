use bevy::{prelude::*, ecs::system::EntityCommands};
use leafwing_input_manager::prelude::*;
use crate::{prelude::*, blob::*};

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
    asset_server: Res<AssetServer> // used to access files stored in the assets folder.
) { 
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

    // generate two example blobs, one that is controlled
    spawn_blob(
        &mut commands,
        gen_l_body(),
        "L Stone",
        Vec3::new(50.0, 300.0, 0.0),
        &|ec| {add_tetris_control(ec);}
    );
    spawn_blob(
        &mut commands,
        gen_t_body(),
        "T Stone",
        Vec3::new(178.0, 300.0, 0.0),
        &|_| {}
    );
    
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

pub fn apply_powerups_by_actions(
    mut query: Query<(&ActionState<WASDActions>, &mut UpgradeableMover)>
) {
    for (a, mut um) in query.iter_mut() {
        if a.just_released(WASDActions::Powerup) {
            um.powerup();
        }
    }
}