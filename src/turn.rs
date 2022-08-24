use bevy::prelude::*;

#[derive(Default)]
pub struct Turn {
    cur: f32,

    new_turn: bool,

    time_per_turn: f32,

    num_turn: i32,

    pub pause: bool,

    pub num_additional_bricks: i32,

    pub fac_id: Option<Entity>,

    pub prod_id: Option<Entity>,
}

impl Turn {
    pub fn new(time_per_turn: f32) -> Self {
        Turn {
            cur: 0.0,
            time_per_turn,
            new_turn: false,
            num_turn: 0,
            num_additional_bricks: 0,
            fac_id: None,
            prod_id: None,
            pause: false,
        }
    }

    pub fn apply_time(&mut self, dt: f32) {
        if self.pause {
            return;
        }
        //~

        self.new_turn = false;
        self.cur += dt;
        if self.cur > self.time_per_turn {
            self.cur -= self.time_per_turn;
            self.new_turn = true;
            self.num_turn += 1;
        }
    }

    pub fn is_new_turn(&self) -> bool {
        self.new_turn && !self.pause
    }
    pub fn get_num_turn(&self) -> i32 {
        self.num_turn
    }
}

pub fn progress_turn(mut turn_info: ResMut<Turn>, time: ResMut<Time>) {
    turn_info.apply_time(time.delta_seconds());
}
