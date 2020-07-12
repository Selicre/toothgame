use crate::vec2::{Vec2, vec2};

use crate::foreground::Foreground;
use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::state::level::LevelState;

mod player;
mod collision;

#[derive(Copy, Clone)]
pub enum EntityKind {
    Key {
        picked_up: bool,
        unlocking_id: Option<usize>,
        unlock_timer: i32
    },
    Lock,
    Explosion {
        time_left: i32
    },
    Star {
        time_left: i32
    }
}

#[derive(Copy, Clone)]
pub struct EntityData {
    pub pos: Vec2<i32>,
    pub vel: Vec2<i32>,
    pub hitbox: Vec2<i32>,
    pub blocked_by: [bool; 4], // udlr
    pub on_ground: bool,
    pub hflip: bool,
    pub frame: i32,
    pub angle: i32,
}

impl EntityData {
    pub const fn new() -> Self {
        Self {
            pos: vec2(0,0),
            vel: vec2(0,0),
            hitbox: vec2(16, 16),
            blocked_by: [false; 4],
            on_ground: true,
            hflip: false,
            frame: 0,
            angle: 0,
        }
    }
}

pub fn star(pos: Vec2<i32>) -> Entity {
    let mut data = EntityData::new();
    data.pos = pos;
    data.vel = vec2(256, 0);
    data.frame = 2;
    Entity {
        kind: EntityKind::Star { time_left: 60 * 10 },
        data
    }
}
pub fn explosion(pos: Vec2<i32>) -> Entity {
    let mut data = EntityData::new();
    data.pos = pos;
    data.frame = 4;
    Entity {
        kind: EntityKind::Explosion { time_left: 16 },
        data
    }
}
pub fn key(pos: Vec2<i32>) -> Entity {
    let mut data = EntityData::new();
    data.pos = pos;
    data.frame = 0;
    Entity {
        kind: EntityKind::Key { picked_up: false, unlocking_id: None, unlock_timer: 0 },
        data
    }
}
pub fn lock(pos: Vec2<i32>) -> Entity {
    let mut data = EntityData::new();
    data.pos = pos;
    data.frame = 1;
    Entity {
        kind: EntityKind::Lock,
        data
    }
}

#[derive(Copy, Clone)]
pub struct Entity {
    pub kind: EntityKind,
    pub data: EntityData
}
impl Entity {
    pub fn run(&mut self, parent: *mut LevelState) -> bool {
        match &mut self.kind {
            EntityKind::Star { ref mut time_left } => {
                project!(parent.{entity_set, foreground});
                self.data.vel.y += 0x30;
                if *time_left > 60 || *time_left / 2 % 2 == 0 {
                    self.data.frame = 2;
                } else {
                    self.data.frame = 0xFF;
                }
                self.data.hitbox = vec2(12, 12);
                self.data.process_collision(foreground);
                if self.data.blocked_by[0] {
                    //self.data.vel.y = 4096;
                }
                if self.data.blocked_by[1] {
                    self.data.vel.y = -1280;
                }
                if self.data.blocked_by[2] {
                    self.data.vel.x = 256;
                }
                if self.data.blocked_by[3] {
                    self.data.vel.x = -256;
                }
                *time_left -= 1;
                if *time_left == 0 {
                    entity_set.spawn(explosion(self.data.pos));
                }
                *time_left == 0
            }
            EntityKind::Key { ref mut picked_up, ref mut unlocking_id, ref mut unlock_timer } => {
                project!(parent.{entity_set, foreground});
                if *picked_up {
                    let pos = if let Some(id) = unlocking_id {
                        entity_set.list[*id].unwrap().data.pos
                    } else {
                        entity_set.player.pos()
                    };
                    let target = (pos - self.data.pos) / 8;
                    self.data.vel = (self.data.vel * 7 + target) / 8;
                    /*self.data.vel = self.data.vel.map(|c| {
                        if c >= 32 {
                            c - 32
                        } else if c < -32 {
                            c + 32
                        } else {
                            0
                        }
                    });*/
                    if let Some(id) = unlocking_id {
                        *unlock_timer -= 1;
                        if *unlock_timer == 0 {
                            entity_set.list[*id] = None;
                            for i in 0..3 {
                                *foreground.block_at_mut(self.data.pos / 256 / 16 + vec2(0, i-1)) = 0;
                                entity_set.spawn(explosion(self.data.pos + vec2(0, i-1) * 256 * 16));
                            }
                            return true;
                        }
                    } else {
                        for (idx, i) in entity_set.list.iter_mut().enumerate() {
                            if let Some(i) = i {
                                if let EntityKind::Lock = &mut i.kind {
                                    if *unlock_timer != 0 { continue; }
                                    let delta = (i.data.pos - self.data.pos) / 256;
                                    if delta.x.abs() < 32 && delta.y.abs() < 32 {
                                        *unlock_timer = 60;
                                        *unlocking_id = Some(idx);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let delta = (entity_set.player.pos() - self.data.pos) / 256;
                    if delta.x.abs() < 16 && delta.y.abs() < 16 { *picked_up = true; }
                }
                self.data.pos = self.data.pos + self.data.vel;
                false
            }
            EntityKind::Explosion { ref mut time_left } => {
                *time_left -= 1;
                self.data.frame = 8 - (*time_left / 4);
                *time_left == 0
            }
            EntityKind::Lock => {
                false
            }
        }
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        use crate::graphics::MISC;
        let pal = MISC.get_pal();
        let data = MISC.get_data();

        let mut offset = vec2(16,16);
        offset.x /= 2;
        let pos = self.data.pos / 256 - camera - offset;
        let frame = self.data.frame as usize;
        for x in 0..16 {
            let pos_x = if !self.data.hflip { x } else { 15 - x };
            for y in 0..16 {
                if let Some(px) = into.pixel(pos + vec2(x, y + 2)) {
                    let offset = frame * 256;
                    let p = pal[data[(pos_x as i32 + y * 16) as usize + offset] as usize];
                    if p != 0 { *px = p; }
                }
            }
        }

    }
}

pub struct EntitySet {
    pub list: [Option<Entity>; 64],
    pub player: player::Player,
}

impl EntitySet {
    pub fn new() -> Self {
        EntitySet {
            list: [None; 64],
            player: player::Player::new()
        }
    }
    pub fn run(&mut self, parent: *mut LevelState) {
        for (i,c) in self.list.iter_mut().enumerate() {
            if let Some(x) = c {
                let remove = x.run(parent);
                if remove { *c = None; }
            }
        }
        self.player.run(parent);
    }
    pub fn spawn(&mut self, entity: Entity) {
        for i in self.list.iter_mut() {
            if i.is_none() {
                *i = Some(entity);
                break
            }
        }
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        for c in self.list.iter() {
            if let Some(c) = c {
                c.render(camera, into);
            }
        }
        self.player.render(camera, into);
    }
}


