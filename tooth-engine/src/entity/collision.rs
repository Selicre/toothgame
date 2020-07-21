use super::{Entity, EntityData};
use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};
use crate::foreground::Foreground;
use crate::foreground::Solidity;
use crate::controller::Buttons;
use crate::state::level::LevelState;

impl EntityData {
    pub fn collide(&mut self, sensor_loc: Vec2<i32>, foreground: &mut Foreground) -> Solidity {
        use Solidity::*;
        let sensor = foreground.solidity_at(sensor_loc / 16);
        match sensor {
            Coin => {
                use crate::state::get;
                use crate::entity;
                let state = unsafe { get() }.unwrap_level();
                state.entity_set.spawn(entity::explosion((sensor_loc / 16 * 16 + vec2(8, 14)) * 256));
                state.data.coins += 1;
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
        let block_y = sensor_loc.y * 256 & 0x7FFFF000;
        match sensor {
            Solid => {
                self.angle = 0;
                return Some(block_y - 1 * 256);
            }
            EjectUp => {
                self.angle = 0;
                return Some(block_y - 1 * 256);
            },
            Semisolid if self.pos.y <= block_y => {
                self.angle = 0;
                return Some(block_y - 1 * 256);
            }
            HurtTop => {
                self.vel.y = -2048;
                return Some(block_y - 1 * 256);
            }
            Slab if sensor_loc.y % 16 > 7 => {
                return Some(block_y + 7 * 256);
            }
            SlopeAssist { direction: v, steep } /*if if v { self.vel.x < 0 } else { self.vel.x > 0 }*/ => {
                self.angle = if v { 1 } else { -1 } * if steep { 2 } else { 1 };
                let px = -2;
                return Some(block_y + px * 256);
            }
            SlopeSteep(v) if v ^ (sensor_id != 0) => {
                let inside_tile = sensor_loc % 16;
                if v {
                    if inside_tile.y >= inside_tile.x - 1 || (self.on_ground && self.vel.y >= 0) {
                        self.angle = 2;
                        return Some(block_y + (inside_tile.x - 1) * 256);
                    }
                } else {
                    if inside_tile.y >= 15 - inside_tile.x || (self.on_ground && self.vel.y >= 0) {
                        self.angle = -2;
                        return Some(block_y + (15 - inside_tile.x) * 256);
                    }
                }
            }
            SlopeLow(v) if v ^ (sensor_id != 0) => {
                let inside_tile = sensor_loc % 16;
                if v {
                    if inside_tile.y >= inside_tile.x/2 + 7 || (self.on_ground && self.vel.y >= 0) {
                        self.angle = 1;
                        return Some(block_y + (inside_tile.x/2 + 7) * 256);
                    }
                } else {
                    if inside_tile.y >= 14 - inside_tile.x/2 || (self.on_ground && self.vel.y >= 0) {
                        self.angle = -1;
                        return Some(block_y + (14 - inside_tile.x/2) * 256);
                    }
                }
            }
            SlopeHigh(v) if v ^ (sensor_id != 0) => {
                let inside_tile = sensor_loc % 16;
                if v {
                    if inside_tile.y >= inside_tile.x/2 - 1 || (self.on_ground && self.vel.y >= 0) {
                        self.angle = 1;
                        return Some(block_y + (inside_tile.x/2 - 1) * 256);
                    }
                } else {
                    if inside_tile.y >= 6 - inside_tile.x/2 || (self.on_ground && self.vel.y >= 0) {
                        self.angle = -1;
                        return Some(block_y + (6 - inside_tile.x/2) * 256);
                    }
                }
            }
            _ => {}
        }
        None
    }
    pub fn sensor_up(&mut self, sensor_loc: Vec2<i32>, _sensor_id: usize, foreground: &mut Foreground) -> Option<i32> {
        use Solidity::*;
        let sensor = self.collide(sensor_loc, foreground);
        match sensor {
            Solid | Slab => {
                return Some((sensor_loc.y * 256 & 0x7FFFF000) + (16 + self.hitbox.y) * 256);
            }
            _ => {}
        }
        None
    }
    pub fn sensor_side(&mut self, sensor_loc: Vec2<i32>, foreground: &mut Foreground, is_right: bool) -> Option<i32> {
        use Solidity::*;
        let sensor = self.collide(sensor_loc, foreground);
        let loc = sensor_loc.x * 256 & 0x7FFFF000;
        let loc_clamp = if is_right {
            loc + (-1 - self.hitbox.x / 2) * 256
        } else {
            loc + (16 + self.hitbox.x / 2) * 256
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
    // 14093
    pub fn process_collision(&mut self, foreground: &mut Foreground) {
        self.blocked_by = [false; 4];
        // Apply horizontal momentum
        let mut next_pos = vec2(self.pos.x + self.vel.x, self.pos.y);
        let sensor_x_pos = if self.vel.x >= 0 {
            self.hitbox.x / 2 + 1
        } else {
            -self.hitbox.x / 2 - 1
        };
        let sensor_locs = [
            (next_pos / 256 + vec2(sensor_x_pos, -self.hitbox.y)),
            (next_pos / 256 + vec2(sensor_x_pos, -self.hitbox.y / 2)),
            (next_pos / 256 + vec2(sensor_x_pos, 0))
        ];
        let is_right = self.vel.x >= 0;
        let mut res = None;
        for (_idx, i) in sensor_locs.iter().enumerate() {
            let l = self.sensor_side(*i, foreground, is_right);
            res = clamp_opt(l, res, false);
        }
        if let Some(c) = res {
            //self.p_speed = false;
            self.blocked_by[2 + is_right as usize] = true;
            self.vel.x = 0;
            next_pos.x = c
        }
        self.pos = next_pos;

        let mut next_pos = vec2(self.pos.x, self.pos.y + self.vel.y);
        if self.on_ground {
            // If on the ground, shift self according to angle
            next_pos.y += (self.vel.x/2) * self.angle;
            if self.angle != 0 { next_pos.y += 256; }  // HACK: this improves sticking to slopes
        }

        let sensor_y_pos = if self.vel.y >= 0 {
            1
        } else {
            -self.hitbox.y - 1
        };
        let sensor_locs = [
            (next_pos / 256 + vec2(-self.hitbox.x / 2, sensor_y_pos)),
            (next_pos / 256 + vec2( self.hitbox.x / 2, sensor_y_pos))
        ];
        let is_down = self.vel.y >= 0;
        let mut res = None;
        for (id, i) in sensor_locs.iter().enumerate() {
            let l = if is_down {
                self.sensor_down(*i, id, foreground)
            } else {
                self.sensor_up(*i, id, foreground)
            };
            res = clamp_opt(l, res, true);
        }
        if let Some(c) = res {
            next_pos.y = c;
            self.vel.y = 0;
            self.blocked_by[0 + is_down as usize] = true;
        }
        self.on_ground = self.blocked_by[1];
        self.pos = next_pos;
        self.pos.x = self.pos.x.max(self.hitbox.x / 2 * 256);
    }
}


fn clamp_opt<T: Ord>(l: Option<T>, r: Option<T>, is_max: bool) -> Option<T> {
    match (l,r) {
        (None, None) => None,
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (Some(l), Some(r)) => Some(if is_max { l.min(r) } else { l.max(r) })
    }
}
