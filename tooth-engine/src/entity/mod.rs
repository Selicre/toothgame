use crate::vec2::{Vec2, vec2};

mod player;

pub enum EntityKind {
    Key(u8),
    Lock(u8)
}

pub struct EntityData {
    pub pos: Vec2<i32>,
    pub vel: Vec2<i32>,
    pub on_ground: bool,
    pub hflip: bool,
    pub frame: i32
}
