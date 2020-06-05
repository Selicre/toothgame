use super::GameState;

use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::vec2::{Vec2, vec2};
use crate::foreground::Foreground;
use crate::terrain;

pub struct State {
    bg_pos: Vec2<i32>,
    fg_pos: Vec2<i32>,
    level_size: Vec2<i32>,
    foreground: Foreground,
}

impl State {
    pub fn new() -> Self {
        use crate::graphics;
        let mut foreground = Foreground::new();
        foreground.unpack_gfx(0);
        terrain::decode_chunk(vec2(0, 0), foreground.blocks_mut(), &[0x00, 0x3C, 0x01, 0xA7, 0x51, 0x01, 0xA5, 0x52]);
        //*foreground.map_entry(vec2(0, 0)) = 0;
        //*foreground.map_entry(vec2(0, 1)) = 0x32 | 0x80;
        State {
            bg_pos: vec2(0,0),
            fg_pos: vec2(0,0),
            level_size: vec2(768, 256),
            foreground,
        }
    }
    pub fn run(&mut self, fb: &mut Framebuffer, buttons: Buttons) -> Option<GameState> {
        let speed = if buttons.b() { 256 } else if buttons.a() { 16 } else { 4 };
        if buttons.left()  { self.fg_pos.x -= speed; }
        if buttons.right() { self.fg_pos.x += speed; }
        if buttons.up()    { self.fg_pos.y -= speed; }
        if buttons.down()  { self.fg_pos.y += speed; }
        self.fg_pos = self.fg_pos.map(|c| c.max(0));
        self.fg_pos.x = self.fg_pos.x.min(self.level_size.x - 320);
        self.fg_pos.y = self.fg_pos.y.min(self.level_size.y - 180);
        self.foreground.render(self.fg_pos, fb);
        None
    }
}
