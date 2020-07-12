use super::{Entity, EntityData};
use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};
use crate::foreground::Foreground;
use crate::foreground::Solidity;
use crate::controller::Buttons;
use crate::state::level::LevelState;

pub struct Player {
    data: EntityData,
    anim_timer: i32,
    coyote_time: i32,
    p_meter: i32,
    p_speed: bool,
    angle: i32,
    debug_enabled: bool,
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
            coyote_time: 4,     // Time when you can still jump off the ground. WIP
            p_meter: 0,
            p_speed: false,
            angle: 0,
            debug_enabled: false,
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
        let block_y = (sensor_loc.y * 256 & 0x7FFFF000);
        match sensor {
            Solid => {
                self.angle = 0;
                return Some(block_y - 1 * 256);
            }
            EjectUp => {
                self.angle = 0;
                return Some(block_y - 1 * 256);
            },
            Semisolid if self.data.pos.y <= block_y => {
                self.angle = 0;
                return Some(block_y - 1 * 256);
            }
            HurtTop => {
                self.data.vel.y = -2048;
                return Some(block_y - 1 * 256);
            }
            Slab if sensor_loc.y % 16 > 7 => {
                return Some(block_y + 7 * 256);
            }
            SlopeAssist { direction: v, steep } /*if if v { self.data.vel.x < 0 } else { self.data.vel.x > 0 }*/ => {
                let inside_tile = sensor_loc % 16;
                self.angle = if v { 1 } else { -1 } * if steep { 2 } else { 1 };
                let px = -2;
                return Some(block_y + px * 256);
            }
            SlopeSteep(v) if v ^ (sensor_id != 0) => {
                let inside_tile = sensor_loc % 16;
                if v {
                    if inside_tile.y >= inside_tile.x - 1 || (self.data.on_ground && self.data.vel.y >= 0) {
                        self.angle = 2;
                        return Some(block_y + (inside_tile.x - 1) * 256);
                    }
                } else {
                    if inside_tile.y >= 15 - inside_tile.x || (self.data.on_ground && self.data.vel.y >= 0) {
                        self.angle = -2;
                        return Some(block_y + (15 - inside_tile.x) * 256);
                    }
                }
            }
            SlopeLow(v) if v ^ (sensor_id != 0) => {
                let inside_tile = sensor_loc % 16;
                if v {
                    if inside_tile.y >= inside_tile.x/2 + 7 || (self.data.on_ground && self.data.vel.y >= 0) {
                        self.angle = 1;
                        return Some(block_y + (inside_tile.x/2 + 7) * 256);
                    }
                } else {
                    if inside_tile.y >= 14 - inside_tile.x/2 || (self.data.on_ground && self.data.vel.y >= 0) {
                        self.angle = -1;
                        return Some(block_y + (14 - inside_tile.x/2) * 256);
                    }
                }
            }
            SlopeHigh(v) if v ^ (sensor_id != 0) => {
                let inside_tile = sensor_loc % 16;
                if v {
                    if inside_tile.y >= inside_tile.x/2 - 1 || (self.data.on_ground && self.data.vel.y >= 0) {
                        self.angle = 1;
                        return Some(block_y + (inside_tile.x/2 - 1) * 256);
                    }
                } else {
                    if inside_tile.y >= 6 - inside_tile.x/2 || (self.data.on_ground && self.data.vel.y >= 0) {
                        self.angle = -1;
                        return Some(block_y + (6 - inside_tile.x/2) * 256);
                    }
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
                return Some(loc_clamp);
            }
            Slab if sensor_loc.y % 16 > 7 => {
                return Some(loc_clamp);
            }
            SlopeSteep(v) if v != is_right => {
                // not sure if this is necessary, but I think this can reduce some jank.
                self.angle = 2 * if v { 1 } else { -1 };
            }
            _ => {}
        }
        None
    }
    pub fn run(&mut self, data: &[Option<Entity>], parent: *mut LevelState) {
        project!(parent.{foreground, buttons});
        let data = &mut self.data;
        if buttons.b() {
            let speed = if buttons.c() {
                0x8000
            } else {
                0x0400
            };
            if buttons.left()  { data.pos.x -= speed; }
            if buttons.right() { data.pos.x += speed; }
            if buttons.up()    { data.pos.y -= speed; }
            if buttons.down()  { data.pos.y += speed; }
            return;
        }
        self.debug_enabled ^= buttons.c_edge();
        if !data.on_ground {
            if buttons.a() {
                data.vel.y += 0x30;
            } else {
                data.vel.y += 0x60;
            }
            data.vel.y = data.vel.y.min(1024);
            self.angle = 0;
        } else {
            if buttons.a_edge() {
                let lift = 0x500 + ((data.vel.x.abs() / 0x80) * 10 / 4) * 0x10;
                data.vel.y = -lift + 0x30;
                data.on_ground = false;
                self.coyote_time = 0;
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
                data.frame = 0;
            }
        } else {
            data.frame = 1;
        }
        // Apply horizontal momentum
        let mut next_pos = vec2(self.data.pos.x + self.data.vel.x, self.data.pos.y);
        if self.data.vel.x >= 0 {
            // Collide with right wall
            let sensor_locs = [
                (next_pos / 256 + vec2(Self::HITBOX.x / 2 + 1, -Self::HITBOX.y)),
                (next_pos / 256 + vec2(Self::HITBOX.x / 2 + 1, -Self::HITBOX.y / 2)),
                (next_pos / 256 + vec2(Self::HITBOX.x / 2 + 1, 0))
            ];
            self.debug_sensors[2..].copy_from_slice(&sensor_locs);
            let mut res = None;
            for i in sensor_locs.iter() {
                let l = self.sensor_side(*i, foreground, true);
                res = clamp_opt(l, res, false);
            }
            if let Some(c) = res {
                self.p_speed = false;
                self.data.vel.x = 0;
                next_pos.x = c
            }
        } else {
            // Collide with left wall
            let sensor_locs = [
                (next_pos / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y)),
                (next_pos / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y / 2)),
                (next_pos / 256 + vec2(-Self::HITBOX.x / 2 - 1, 0))
            ];
            self.debug_sensors[2..].copy_from_slice(&sensor_locs);
            let mut res = None;
            for i in sensor_locs.iter() {
                let l = self.sensor_side(*i, foreground, false);
                res = clamp_opt(l, res, true);
            }
            if let Some(c) = res {
                self.p_speed = false;
                self.data.vel.x = 0;
                next_pos.x = c
            }
        }
        self.data.pos = next_pos;

        let mut next_pos = vec2(self.data.pos.x, self.data.pos.y + self.data.vel.y);
        if self.data.on_ground {
            // If on the ground, shift self
            next_pos.y += (self.data.vel.x/2) * self.angle;
            if self.angle != 0 { next_pos.y += 256; }  // HACK: this improves sticking to slopes
        }
        if self.data.vel.y >= 0 {
            // Collide with ground
            let sensor_locs = [
                (next_pos / 256 + vec2(-Self::HITBOX.x / 2, 1)),
                (next_pos / 256 + vec2( Self::HITBOX.x / 2, 1))
            ];
            self.debug_sensors[..2].copy_from_slice(&sensor_locs);
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
            // Collide with ceiling
            let sensor_locs = [
                (next_pos / 256 + vec2(-Self::HITBOX.x / 2, -Self::HITBOX.y - 1)),
                (next_pos / 256 + vec2( Self::HITBOX.x / 2, -Self::HITBOX.y - 1))
            ];
            self.debug_sensors[..2].copy_from_slice(&sensor_locs);
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
        self.data.pos = next_pos;
        self.data.pos.x = self.data.pos.x.max(Self::HITBOX.x / 2 * 256);
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
                if let Some(px) = into.pixel(pos + vec2(x, y + 2)) {
                    let offset = if y >= 16 { 3 * 256 + frame * 256 } else { 0 * 256 + frame * 256 };
                    let p = pal[data[(pos_x as i32 + (y % 16) * 16) as usize + offset] as usize];
                    if p != 0 { *px = p; }
                }
            }
        }

        if self.debug_enabled {
            for (pos,i) in into.pixels() {
                let pos = pos + camera;
                let block_offset = pos & 15;
                if block_offset.x == 0 || block_offset.y == 0 {
                    *i &= 0x7FFFFFFF;
                }
            }
            let color = if self.p_speed { 0xFF0000FF } else { 0xFFFFFFFF };
            for i in 0..self.p_meter {
                for y in 0..4 {
                    into.pixel(vec2(16 + i, 16 + y)).map(|c| *c = color);
                }
            }
            for i in 0..16 {
                let color = match self.angle {
                    -2 => 0xFF0000,
                    -1 => 0xFF8080,
                     0 => 0xFFFFFF,
                     1 => 0x8080FF,
                     2 => 0x0000FF,
                     _ => unreachable!()
                };
                into.pixel(vec2(16 + i%4, 8 + i/4)).map(|c| *c = 0xFF000000 + color);
                if self.data.on_ground {
                    into.pixel(vec2(20 + i%4, 8 + i/4)).map(|c| *c = 0xFFFFFFFF);
                }
            }
            into.pixel(self.data.pos / 256 - camera).map(|c| *c = 0xFF0000FF);
            for i in self.debug_sensors.iter() {
                into.pixel(*i - camera).map(|c| *c = 0xFFFF0000);
            }
        }
        /*
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
