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

#[derive(Copy, Clone)]
pub struct Entity {
    kind: EntityKind,
    data: EntityData
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

pub struct EntitySet {
    pub data: [Option<Entity>; 64],
    pub player: player::Player
}

impl EntitySet {
    pub fn new() -> Self {
        EntitySet {
            data: [None; 64],
            player: player::Player::new()
        }
    }
    pub fn run(&mut self, buttons: Buttons, foreground: &mut Foreground) {
        self.player.run(buttons, &self.data, foreground);
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        self.player.render(camera, into);
    }
}


