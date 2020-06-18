use super::{Entity, EntityData};
use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};
use crate::foreground::Foreground;
use crate::controller::Buttons;

pub struct Player {
    data: EntityData,
    anim_timer: i32,
    p_meter: i32,
    p_speed: bool,
    debug_sensors: [Vec2<i32>; 5]
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
            debug_sensors: [vec2(0,0); 5]
        }
    }
    pub fn pos(&self) -> Vec2<i32> {
        self.data.pos / 256
    }
    pub fn set_pos(&mut self, pos: Vec2<i32>) {
        self.data.pos = pos * 256;
    }
    pub fn run(&mut self, buttons: Buttons, data: &[Option<Entity>], foreground: &mut Foreground) {
        let data = &mut self.data;
        if buttons.c_edge() {
            data.pos.y += 0x1000;
        }
        if !data.on_ground {
            if buttons.a() {
                data.vel.y += 0x30;
            } else {
                data.vel.y += 0x60;
            }

            data.vel.y = data.vel.y.min(1024);
        } else {
            if buttons.a_edge() {
                let lift = [0x500, 0x520, 0x550, 0x570, 0x5A0, 0x5C0, 0x5F0, 0x610][data.vel.x.abs() as usize / 0x80];
                data.vel.y = -lift + 0x30;
                data.on_ground = false;
            } else {
                data.vel.y = 0;
            }
        }
        let max_speed = if self.p_meter == 0x70 { 0x300 } else { 0x240 };
        if buttons.left() {
            data.hflip = true;
            if data.vel.x >= 0 {
                data.vel.x = (data.vel.x - 0x50).max(-max_speed);
            } else {
                data.vel.x = (data.vel.x - 0x18).max(-max_speed);
                if data.on_ground || self.p_speed { self.p_meter += 3; }
            }
        } else if buttons.right() {
            data.hflip = false;
            if data.vel.x > 0 {
                data.vel.x = (data.vel.x + 0x18).min(max_speed);
                if data.on_ground || self.p_speed { self.p_meter += 3; }
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
        let next_pos_x = vec2(next_pos.x, data.pos.y);
        if data.vel.x >= 0 {
            // Collide with right wall
            let sensor_top_loc = (next_pos_x / 256 + vec2(Self::HITBOX.x / 2 + 1, -Self::HITBOX.y));
            let sensor_mid_loc = (next_pos_x / 256 + vec2(Self::HITBOX.x / 2 + 1, -Self::HITBOX.y / 2));
            let sensor_bot_loc = (next_pos_x / 256 + vec2(Self::HITBOX.x / 2 + 1, 0));
            self.debug_sensors[2] = sensor_top_loc;
            self.debug_sensors[3] = sensor_mid_loc;
            self.debug_sensors[4] = sensor_bot_loc;
            let sensor_top = foreground.solidity_at(sensor_top_loc / 16);
            let sensor_mid = foreground.solidity_at(sensor_mid_loc / 16);
            let sensor_bot = foreground.solidity_at(sensor_bot_loc / 16);
            if !(sensor_top == 0 && sensor_mid == 0 && sensor_bot == 0) {
                self.p_speed = false;
                data.vel.x = 0;
                next_pos.x = (sensor_top_loc.x * 256 & 0x7FFFF000) + (-1 - Self::HITBOX.x / 2) * 256
            }
        } else {
            // Collide with left wall
            let sensor_top_loc = (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y));
            let sensor_mid_loc = (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, -Self::HITBOX.y / 2));
            let sensor_bot_loc = (next_pos_x / 256 + vec2(-Self::HITBOX.x / 2 - 1, 0));
            self.debug_sensors[2] = sensor_top_loc;
            self.debug_sensors[3] = sensor_mid_loc;
            self.debug_sensors[4] = sensor_bot_loc;
            let sensor_top = foreground.solidity_at(sensor_top_loc / 16);
            let sensor_mid = foreground.solidity_at(sensor_mid_loc / 16);
            let sensor_bot = foreground.solidity_at(sensor_bot_loc / 16);
            if !(sensor_top == 0 && sensor_mid == 0 && sensor_bot == 0) {
                self.p_speed = false;
                data.vel.x = 0;
                next_pos.x = (sensor_top_loc.x * 256 & 0x7FFFF000) + (16 + Self::HITBOX.x / 2) * 256
            }
        }
        let next_pos_y = next_pos;
        if data.vel.y >= 0 {
            // Collide with ground
            let sensor_l_loc = (next_pos_y / 256 + vec2(-Self::HITBOX.x / 2, 1));
            let sensor_r_loc = (next_pos_y / 256 + vec2( Self::HITBOX.x / 2, 1));
            self.debug_sensors[0] = sensor_l_loc;
            self.debug_sensors[1] = sensor_r_loc;
            let sensor_l = foreground.solidity_at(sensor_l_loc / 16);
            let sensor_r = foreground.solidity_at(sensor_r_loc / 16);
            if sensor_l == 0 && sensor_r == 0 {
                data.on_ground = false;
            } else {
                data.on_ground = true;
                next_pos.y = (sensor_l_loc.y * 256 & 0x7FFFF000) - 1 * 256;
            }
        } else {
            // Collide with ceiling
            let sensor_l_loc = (next_pos_y / 256 + vec2(-Self::HITBOX.x / 2, -Self::HITBOX.y - 1));
            let sensor_r_loc = (next_pos_y / 256 + vec2( Self::HITBOX.x / 2, -Self::HITBOX.y - 1));
            self.debug_sensors[0] = sensor_l_loc;
            self.debug_sensors[1] = sensor_r_loc;
            let sensor_l = foreground.solidity_at(sensor_l_loc / 16);
            let sensor_r = foreground.solidity_at(sensor_r_loc / 16);
            if !(sensor_l == 0 && sensor_r == 0) {
                data.vel.y = 0;
                next_pos.y = (sensor_l_loc.y * 256 & 0x7FFFF000) + (16 + Self::HITBOX.y) * 256;
            }
        }
        next_pos.x = next_pos.x.max(Self::HITBOX.x / 2 * 256);
        data.pos = next_pos;
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
