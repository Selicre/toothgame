use crate::vec2::{Vec2, vec2};

use crate::foreground::Foreground;
use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::state::level::LevelState;

mod player;

#[derive(Copy, Clone)]
pub enum EntityKind {
    //Key(u8),
    //Lock(u8),
    Star
}

#[derive(Copy, Clone)]
pub struct EntityData {
    pub pos: Vec2<i32>,
    pub vel: Vec2<i32>,
    pub on_ground: bool,
    pub hflip: bool,
    pub frame: i32
}

impl EntityData {
    pub const fn new() -> Self {
        Self {
            pos: vec2(0,0),
            vel: vec2(0,0),
            on_ground: true,
            hflip: false,
            frame: 0
        }
    }
}

#[derive(Copy, Clone)]
pub struct Entity {
    pub kind: EntityKind,
    pub data: EntityData
}
impl Entity {
    pub fn run(&mut self, others: &EntitySetCopy, parent: *mut LevelState) {
        match self.kind {
            EntityKind::Star => {
                project!(parent.entity_set.player as player);
                self.data.pos = player.pos() * 256 - vec2(0, 50 * 256);
                self.data.frame = 2;
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
    pub player: player::Player
}

pub struct EntitySetCopy {
    list: [Option<Entity>; 64],
    current: usize
}

impl EntitySetCopy {
    pub fn iter(&self) -> impl Iterator<Item=&Entity> + '_ {
        self.list.iter().enumerate()
            .filter(move |(i,_)| *i == self.current)
            .filter_map(|(_,c)| c.as_ref())
    }
    pub fn get(&self, idx: usize) -> Option<&Entity> {
        self.list[idx].as_ref()
    }
}

impl EntitySet {
    pub fn new() -> Self {
        EntitySet {
            list: [None; 64],
            player: player::Player::new()
        }
    }
    pub fn run(&mut self, parent: *mut LevelState) {
        let mut list_copy = EntitySetCopy {
            list: self.list,
            current: 0
        };
        for (i,c) in self.list.iter_mut().enumerate() {
            if let Some(c) = c {
                list_copy.current = i;
                c.run(&list_copy, parent);
            }
        }
        self.player.run(&self.list, parent);
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


