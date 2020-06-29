use super::{Entity, EntityData};
use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};
use crate::foreground::Foreground;
use crate::foreground::Solidity;
use crate::controller::Buttons;

pub struct Player {
    data: EntityData,
    anim_timer: i32,
    p_meter: i32,
    p_speed: bool,
    angle: i32,
    debug_sensors: [Vec2<i32>; 5],
    debug_lines: [i32;2],
}

fn clamp_opt<T: Ord>(l: Option<T>, r: Option<T>, is_max: bool) -> Option<T> {
    match (l,r) {
        (None, None) => None,
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (Some(l), Some(r)) => Some(if is_max { l.min(r) } else { l.max(r) })
    }
}
enum Direction {
    Up, Down, Left, Right
}
impl Player {
    pub const HITBOX: Vec2<i32> = vec2(10, 24);
    pub const SPRITE_SIZE: Vec2<i32> = vec2(16, 32);
    pub const fn new() -> Self {
        Player {
            data: EntityData::new(),
            anim_timer: 0,
            p_meter: 0,
            p_speed: false,
            angle: 0,
            debug_sensors: [vec2(0,0); 5],
            debug_lines: [0; 2]
        }
    }
    pub fn pos(&self) -> Vec2<i32> {
        self.data.pos / 256
    }
    pub fn set_pos(&mut self, pos: Vec2<i32>) {
        self.data.pos = pos * 256;
    }
    pub fn collide(&mut self, sensor_loc: Vec2<i32>, foreground: &mut Foreground) -> Solidity {
        use Solidity::*;
        let sensor = foreground.solidity_at(sensor_loc / 16);
        match sensor {
            Coin => {
                let block = foreground.block_at_mut(sensor_loc / 16);
                *block = 0;
            },
            _ => {}
        }
        sensor
    }
    pub fn sensor_down(&mut self, sensor_loc: Vec2<i32>, sensor_id: usize, foreground: &mut Foreground) -> Option<i32> {
        use Solidity::*;
        let sensor = self.collide(sensor_loc, foreground);
        match sensor {
            Solid | Semisolid => {
                self.angle = 0;
                return Some((sensor_loc.y * 256 & 0x7FFFF000) - 1 * 256);
            }
            HurtTop => {
                self.angle = 0;
                self.data.vel.y = -2048;
                return Some((sensor_loc.y * 256 & 0x7FFFF000) - 1 * 256);
            }
            Slab if sensor_loc.y % 16 > 7 => {
                self.angle = 0;
                return Some((sensor_loc.y * 256 & 0x7FFFF000) + 7 * 256);
            }
            SlopeAssist if self.data.vel.x < 0 => {
                self.angle = 2;
                return Some((sensor_loc.y * 256 & 0x7FFFF000) - 2 * 256);
            }
            SlopeSteep(v) if sensor_id == 0 => {
                let inside_tile = sensor_loc % 16;
                if inside_tile.y >= (inside_tile.x - 1) {
                    self.angle = 2;
                    return Some((sensor_loc.y * 256 & 0x7FFFF000) + (inside_tile.x - 1) * 256);
                }
            }
            _ => {}
        }
        None
    }
    pub fn sensor_up(&mut self, sensor_loc: Vec2<i32>, foreground: &mut Foreground) -> Option<i32> {
        use Solidity::*;
        let sensor = self.collide(sensor_loc, foreground);
        match sensor {
            Solid | Slab => {
                return Some((sensor_loc.y * 256 & 0x7FFFF000) + (16 + Self::HITBOX.y) * 256);
            }
            _ => {}
        }
        None
    }
    pub fn sensor_side(&mut self, sensor_loc: Vec2<i32>, foreground: &mut Foreground, is_right: bool) -> Option<i32> {
        use Solidity::*;
        let sensor = self.collide(sensor_loc, foreground);
        let loc = (sensor_loc.x * 256 & 0x7FFFF000);
        let loc_clamp = if is_right {
            loc + (-1 - Self::HITBOX.x / 2) * 256
        } else {
            loc + (16 + Self::HITBOX.x / 2) * 256
        };
        match sensor {
            Solid | HurtTop => {
                self.p_speed = false;
                return Some(loc_clamp);
            }
            Slab if sensor_loc.y % 16 > 7 => {
                self.p_speed = false;
                return Some(loc_clamp);
            }
            _ => {}
        }
        None
    }
    pub fn run(&mut self, buttons: Buttons, data: &[Option<Entity>], foreground: &mut Foreground) {
        let data = &mut self.data;
        /*if buttons.c_edge() {
            data.pos.y += 0x1000;
        }*/
        if !data.on_ground {
            if buttons.a() {
                data.vel.y += 0x30;
            } else {
                data.vel.y += 0x60;
            }

            data.vel.y = data.vel.y.min(1024);
        } else {
            if buttons.a_edge() {
                let lift = 0x500 + ((data.vel.x.abs() / 0x80) * 10 / 4) * 0x10;
                data.vel.y = -lift + 0x30;
                data.on_ground = false;
            } else {
                if data.vel.y >= 0 {
                    data.vel.y = 0;
                } else {
                    data.on_ground = false;
                }
            }
        }
        let max_speed = if self.p_meter == 0x70 { 0x300 } else { 0x240 };
        if buttons.left() {
            data.hflip = true;
            if data.vel.x >= 0 {
                data.vel.x = (data.vel.x - 0x50).max(-max_speed);
            } else {
                data.vel.x = (data.vel.x - 0x18).max(-max_speed);
                if data.vel.x <= -0x240 && (data.on_ground || self.p_speed) { self.p_meter += 3; }
            }
        } else if buttons.right() {
            data.hflip = false;
            if data.vel.x > 0 {
                data.vel.x = (data.vel.x + 0x18).min(max_speed);
                if data.vel.x >= 0x240 && (data.on_ground || self.p_speed) { self.p_meter += 3; }
            } else {
                data.vel.x = (data.vel.x + 0x50).min(max_speed);
            }
        } else if data.on_ground {
            // if on ground and not pressing any buttons..
            self.p_speed = false;
            if data.vel.x > 0 {
                data.vel.x -= 0x10;
                if data.vel.x < 0 { data.vel.x = 0; }
            } else {
                data.vel.x += 0x10;
                if data.vel.x > 0 { data.vel.x = 0; }
            }
        }
        self.p_meter -= 1;
        if self.p_meter > 0x70 { self.p_speed = true; self.p_meter = 0x70; }
        if self.p_meter < 0 { self.p_meter = 0; }
        if data.on_ground {
            self.anim_timer += data.vel.x.abs();
            if self.anim_timer > 0xA00 {
                self.anim_timer -= 0xA00;
                data.frame += 1;
                data.frame %= 3;
            }
            if data.vel.x == 0 {
                self.anim_timer = 0x9FF;
                data.frame = 2;
            }
        } else {
            data.frame = 1;
        }
        let mut next_pos = data.pos + data.vel;
        if data.on_ground {
            if self.angle == 2 {
                next_pos.y += data.vel.x;
            }
        }
        let next_pos_x = vec2(next_pos.x, data.pos.y);
        if data.vel.x >= 0 {
            use Solidity::*;
            // Collide with right wall
            /*
            //self.debug_sensors[2] = sensor_top_loc;
            //self.debug_sensors[3] = sensor_mid_loc;
            //self.debug_sensors[4] = sensor_bot_loc;
            let sensor_top = foreground.solidity_at(sensor_top_loc / 16);
            let sensor_mid = foreground.solidity_at(sensor_mid_loc / 16);
            let sensor_bot = foreground.solidity_at(sensor_bot_loc / 16);
            if sensor_top == Solid || sensor_mid == Solid || sensor_bot == Solid {
                self.p_speed = false;
                data.vel.x = 0;
                next_pos.x = (sensor_top_loc.x * 256 & 0x7FFFF000) + (-1 - Self::HITBOX.x / 2) * 256
            }*/
            let sensor_locs = [
                (next_pos_x / 256 + vec2(Self::HITBOX.x / 2 + 1, -Self::HITBOX.y)),
                (next_pos_x / 256 + vec2(Self::HITBOX.x / 2 + 1, -Self::HITBOX.y / 2)),
                (next_pos_x / 256 + vec2(Self::HITBOX.x / 2 + 1, 0))
            ];
            let mut res = None;
            for i in sensor_locs.iter() {
                let l = self.sensor_side(*i, foreground, true);
                res = clamp_opt(l, res, false);
            }
            if let Some(c) = res {
                self.data.vel.x = 0;
                next_pos.x = c
            }
        } else {
            use Solidity::*;
            // Collide with left wall
            /*let sensor_top_loc = (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y));
            let sensor_mid_loc = (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y / 2));
            let sensor_bot_loc = (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, 0));
            //self.debug_sensors[2] = sensor_top_loc;
            //self.debug_sensors[3] = sensor_mid_loc;
            //self.debug_sensors[4] = sensor_bot_loc;
            let sensor_top = foreground.solidity_at(sensor_top_loc / 16);
            let sensor_mid = foreground.solidity_at(sensor_mid_loc / 16);
            let sensor_bot = foreground.solidity_at(sensor_bot_loc / 16);
            if sensor_top == Solid || sensor_mid == Solid || sensor_bot == Solid {
                self.p_speed = false;
                data.vel.x = 0;
                next_pos.x = (sensor_top_loc.x * 256 & 0x7FFFF000) + (16 + Self::HITBOX.x / 2) * 256
            }*/
            //9613
            let sensor_locs = [
                (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y)),
                (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y / 2)),
                (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, 0))
            ];
            let mut res = None;
            for i in sensor_locs.iter() {
                let l = self.sensor_side(*i, foreground, false);
                res = clamp_opt(l, res, false);
            }
            if let Some(c) = res {
                self.data.vel.x = 0;
                next_pos.x = c
            }
        }
        let next_pos_y = next_pos;
        if self.data.vel.y >= 0 {
            // Collide with ground
            let sensor_locs = [
                (next_pos_y / 256 + vec2(-Self::HITBOX.x / 2, 1)),
                (next_pos_y / 256 + vec2( Self::HITBOX.x / 2, 1))
            ];
            let mut res = None;
            for (id, i) in sensor_locs.iter().enumerate() {
                let l = self.sensor_down(*i, id, foreground);
                res = clamp_opt(l, res, true);
            }
            self.data.on_ground = res.is_some();
            if let Some(c) = res {
                next_pos.y = c
            }
        } else {
            use Solidity::*;
            // Collide with ceiling
            let sensor_locs = [
                (next_pos_y / 256 + vec2(-Self::HITBOX.x / 2, -Self::HITBOX.y - 1)),
                (next_pos_y / 256 + vec2( Self::HITBOX.x / 2, -Self::HITBOX.y - 1))
            ];
            let mut res = None;
            for i in sensor_locs.iter() {
                let l = self.sensor_up(*i, foreground);
                res = clamp_opt(l, res, false);
            }
            if let Some(c) = res {
                next_pos.y = c;
                self.data.vel.y = 0;
            }
        }
        next_pos.x = next_pos.x.max(Self::HITBOX.x / 2 * 256);
        self.data.pos = next_pos;
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        use crate::graphics::TOOTHPASTE;

        let pal = TOOTHPASTE.get_pal();
        let data = TOOTHPASTE.get_data();

        let mut offset = Self::SPRITE_SIZE;
        offset.x /= 2;
        let pos = self.data.pos / 256 - camera - offset;
        let frame = self.data.frame as usize;
        for x in 0..16 {
            let pos_x = if !self.data.hflip { x } else { 15 - x };
            for y in 0..32 {
                if let Some(px) = into.pixel(pos + vec2(x, y + 1)) {
                    let offset = if y >= 16 { 3 * 256 + frame * 256 } else { 0 * 256 + frame * 256 };
                    let p = pal[data[(pos_x as i32 + (y % 16) * 16) as usize + offset] as usize];
                    if p != 0 { *px = p; }
                }
            }
        }
        let color = if self.p_speed { 0xFF0000FF } else { 0xFFFFFFFF };
        for i in 0..self.p_meter {
            into.pixel(vec2(16 + i, 16)).map(|c| *c = color);
            into.pixel(vec2(16 + i, 17)).map(|c| *c = color);
        }
        if self.angle == 2 {
            into.pixel(vec2(16, 8)).map(|c| *c = 0xFF0000FF);
        }
        /*
        into.pixel(self.data.pos / 256 - camera).map(|c| *c = 0xFF0000FF);
        for i in self.debug_sensors.iter() {
            into.pixel(*i - camera).map(|c| *c = 0xFFFF0000);
        }
        for x in -Self::HITBOX.x / 2 ..= Self::HITBOX.x / 2 {
            for y in -Self::HITBOX.y ..= 0 {
                if let Some(px) = into.pixel(pos + vec2(x, y) + offset) {
                    let [r,g,b,a] = px.to_ne_bytes();
                    let r = r.saturating_sub(0x50);
                    let g = g.saturating_sub(0x50);
                    let b = b.saturating_sub(0x50);
                    *px = u32::from_ne_bytes([r,g,b,a]);
                }
            }
        }
        */
    }
}
