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
}

impl Player {
    pub const HITBOX: Vec2<i32> = vec2(10, 24);
    pub const SPRITE_SIZE: Vec2<i32> = vec2(16, 32);
    pub const fn new() -> Self {
        let mut data = EntityData::new();
        data.hitbox = Self::HITBOX;
        Player {
            data,
            anim_timer: 0,
            coyote_time: 4,     // Time when you can still jump off the ground. WIP
            p_meter: 0,
            p_speed: false,
            angle: 0,
            debug_enabled: false,
            debug_sensors: [vec2(0,0); 5],
        }
    }
    pub fn pos(&self) -> Vec2<i32> {
        self.data.pos
    }
    pub fn set_pos(&mut self, pos: Vec2<i32>) {
        self.data.pos = pos;
    }
    pub fn run(&mut self, parent: *mut LevelState) {
        project!(parent.{foreground, buttons, entity_set});
        let data = &mut self.data;
        if cfg!(feature = "debug") && buttons.start() {
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
        if cfg!(feature = "debug") && buttons.b_edge() {
            use crate::entity::star;
            entity_set.spawn(star(data.pos));
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
        data.process_collision(foreground);
        if data.blocked_by[2] || data.blocked_by[3] { self.p_speed = false; }
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

        if cfg!(feature = "debug") && self.debug_enabled {
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
    }
}
