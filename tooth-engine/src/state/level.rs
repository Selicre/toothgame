use super::GameState;

use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::vec2::{Vec2, vec2};
use crate::foreground::Foreground;
use crate::background::Background;
use crate::entity::EntitySet;
use crate::terrain;

pub struct State {
    camera: Vec2<i32>,
    level_size: Vec2<i32>,
    foreground: Foreground,
    background: Background,
    entity_set: EntitySet,
    fadein_timer: i32,
}

impl State {
    pub fn new() -> Self {
        use crate::graphics;
        let mut background = Background::new();
        let mut foreground = Foreground::new();
        let mut entity_set = EntitySet::new();
        entity_set.player.set_pos(vec2(32, 160));
        terrain::decode_chunk(vec2(0, 0), foreground.blocks_mut(), &[0x00, 0x3C, 0x01, 0xA7, 0x51, 0x01, 0xA5, 0x52]);
        terrain::decode_chunk(vec2(16, 0), foreground.blocks_mut(), &[
            0x01, 0x0B, 0x51, 0x01, 0x1A, 0x41, 0x01, 0x29, 0x31,
            0x01, 0x38, 0x21, 0x01, 0x65, 0x52, 0x01, 0xAA, 0x21
        ]);
        terrain::decode_chunk(vec2(2 * 16, 0), foreground.blocks_mut(), &[
            0x03, 0x05, 0xA1
        ]);
        terrain::decode_chunk(vec2(3 * 16, 0), foreground.blocks_mut(), &[0x00, 0x10]);
        State {
            camera: vec2(0,60),
            level_size: vec2(2048, 256),
            foreground,
            background,
            entity_set,
            fadein_timer: 0
        }
    }
    pub fn run(&mut self, fb: &mut Framebuffer, buttons: Buttons) -> Option<GameState> {
        /*let speed = if buttons.b() { 256 } else if buttons.a() { 16 } else { 4 };
        if buttons.left()  { self.camera.x -= speed; }
        if buttons.right() { self.camera.x += speed; }
        if buttons.up()    { self.camera.y -= speed; }
        if buttons.down()  { self.camera.y += speed; }*/
        self.entity_set.run(buttons, &mut self.foreground);

        let camera_target = self.entity_set.player.pos() - vec2(320 / 2, 180 / 2 + 16);
        //self.camera = (self.camera + camera_target) / 2;

        if camera_target.x - 0x10 > self.camera.x {
            self.camera.x = camera_target.x - 0x10;
        } else if camera_target.x + 0x10 < self.camera.x {
            self.camera.x = camera_target.x + 0x10;
        }
        if camera_target.y - 0x10 > self.camera.y {
            self.camera.y = camera_target.y - 0x10;
        } else if camera_target.y + 0x10 < self.camera.y {
            self.camera.y = camera_target.y + 0x10;
        }

        self.camera = self.camera.map(|c| c.max(0));
        self.camera.x = self.camera.x.min(self.level_size.x - 320);
        self.camera.y = self.camera.y.min(self.level_size.y - 180);

        self.background.render(self.camera, fb);
        self.foreground.render(self.camera, fb);
        self.entity_set.render(self.camera, fb);

        if self.fadein_timer < 320 {
            self.fadein_timer += 8;
            let center = self.entity_set.player.pos() - self.camera - vec2(0, 24);
            for (pos,px) in fb.pixels() {
                let dist = pos - center;
                if dist.x*dist.x + dist.y*dist.y > self.fadein_timer*self.fadein_timer {
                    *px = 0xFF000000;
                }
            }
        }
        None
    }
}
