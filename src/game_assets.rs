use crate::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct GameAssets {
    pub block_blob: Handle<Image>,

    pub block_direction_d: Handle<Image>,

    pub block_direction_l: Handle<Image>,

    pub block_direction_r: Handle<Image>,

    pub block_direction_u: Handle<Image>,

    pub block_factory_floor: Handle<Image>,

    pub block_rotate_l: Handle<Image>,

    pub block_rotate_r: Handle<Image>,

    pub block_target_outline: Handle<Image>,

    pub block_tetris_floor: Handle<Image>,

    pub normal_button_color: Color,

    pub hover_button_color: Color,

    pub selected_button_color: Color,

    pub clicked_button_color: Color,

    pub tool_move_left: Handle<Image>,

    pub tool_move_right: Handle<Image>,

    pub tool_move_up: Handle<Image>,

    pub tool_move_down: Handle<Image>,

    pub tool_rotate_left: Handle<Image>,

    pub tool_rotate_right: Handle<Image>,

    pub tool_play: Handle<Image>,

    pub tool_stop: Handle<Image>,

    pub tool_cutter_line: Handle<Image>,

    pub tool_cutter_l: Handle<Image>,

    pub tool_cutter_inv_l: Handle<Image>,

    pub tool_cutter_square: Handle<Image>,

    pub tool_cutter_small_t: Handle<Image>,

    pub tool_cutter_stairs_l: Handle<Image>,

    pub tool_cutter_stairs_r: Handle<Image>,

    pub font: Handle<Font>,
}

impl GameAssets {
    pub fn new(asset_server: &AssetServer) -> Self {
        GameAssets {
            block_blob: asset_server.load("blocks/64/blob.png"),
            block_direction_d: asset_server.load("blocks/64/direction_d.png"),
            block_direction_l: asset_server.load("blocks/64/direction_l.png"),
            block_direction_r: asset_server.load("blocks/64/direction_r.png"),
            block_direction_u: asset_server.load("blocks/64/direction_u.png"),
            block_factory_floor: asset_server.load("blocks/64/factory_floor.png"),
            block_rotate_l: asset_server.load("blocks/64/rotate_l.png"),
            block_rotate_r: asset_server.load("blocks/64/rotate_r.png"),
            block_target_outline: asset_server.load("blocks/64/target_outline.png"),
            block_tetris_floor: asset_server.load("blocks/64/tetris_floor.png"),
            normal_button_color: Color::rgba(0.15, 0.15, 0.15, 0.25),
            hover_button_color: Color::rgba(0.25, 0.25, 0.25, 0.5),
            selected_button_color: Color::rgba(0.25, 0.55, 0.25, 0.5),
            clicked_button_color: Color::rgba(0.35, 0.75, 0.35, 0.5),
            tool_move_left: asset_server.load("tools/tool_direction_l.png"),
            tool_move_right: asset_server.load("tools/tool_direction_r.png"),
            tool_move_up: asset_server.load("tools/tool_direction_u.png"),
            tool_move_down: asset_server.load("tools/tool_direction_d.png"),
            tool_rotate_left: asset_server.load("tools/tool_rotate_l.png"),
            tool_rotate_right: asset_server.load("tools/tool_rotate_r.png"),
            tool_play: asset_server.load("tools/tool_play.png"),
            tool_stop: asset_server.load("tools/tool_stop.png"),
            tool_cutter_line: asset_server.load("tools/tool_tetris_i.png"),
            tool_cutter_l: asset_server.load("tools/tool_tetris_l.png"),
            tool_cutter_inv_l: asset_server.load("tools/tool_tetris_l_inv.png"),
            tool_cutter_square: asset_server.load("tools/tool_tetris_square.png"),
            tool_cutter_small_t: asset_server.load("tools/tool_tetris_t.png"),
            tool_cutter_stairs_l: asset_server.load("tools/tool_tetris_z_inv.png"),
            tool_cutter_stairs_r: asset_server.load("tools/tool_tetris_z.png"),
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
        }
    }

    pub fn get_tool_image(&self, tool: Tool) -> &Handle<Image> {
        match tool {
            Tool::Move(d) => match d {
                MoveDirection::Up => &self.tool_move_up,
                MoveDirection::Down => &self.tool_move_down,
                MoveDirection::Left => &self.tool_move_left,
                MoveDirection::Right => &self.tool_move_right,
            },
            Tool::Rotate(d) => match d {
                RotateDirection::Left => &self.tool_rotate_left,
                RotateDirection::Right => &self.tool_rotate_right,
            },
            Tool::Cutter(b) => match b {
                TetrisBricks::Square => &self.tool_cutter_square,
                TetrisBricks::Line => &self.tool_cutter_line,
                TetrisBricks::L => &self.tool_cutter_l,
                TetrisBricks::InvL => &self.tool_cutter_inv_l,
                TetrisBricks::StairsL => &self.tool_cutter_stairs_l,
                TetrisBricks::StairsR => &self.tool_cutter_stairs_r,
                TetrisBricks::SmallT => &self.tool_cutter_small_t,
            },
            Tool::Play => &self.tool_play,
            Tool::Stop => &self.tool_stop,
        }
    }
}
