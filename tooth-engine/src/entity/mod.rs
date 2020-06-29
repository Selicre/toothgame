use crate::vec2::{Vec2, vec2};

use crate::foreground::Foreground;
use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;

mod player;

#[derive(Copy, Clone)]
pub enum EntityKind {
    Key(u8),
    Lock(u8)
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
    kind: EntityKind,
    data: EntityData
}

impl Entity {
    pub fn run(&mut self, list: &[Option<Entity>], foreground: &mut Foreground) {

    }
}

pub struct EntitySet {
    pub list: [Option<Entity>; 64],
    pub player: player::Player
}

impl EntitySet {
    pub fn new() -> Self {
        EntitySet {
            list: [None; 64],
            player: player::Player::new()
        }
    }
    pub fn run(&mut self, buttons: Buttons, foreground: &mut Foreground) {
        let list_copy = self.list;
        for i in self.list.iter_mut().filter_map(|c| c.as_mut()) {
            i.run(&list_copy, foreground);
        }
        self.player.run(buttons, &self.list, foreground);
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        self.player.render(camera, into);
    }
}


