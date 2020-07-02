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
        entity_set.player.set_pos(vec2(1480, 60));
        terrain::decode_chunk(vec2(0, 0), foreground.blocks_mut(), &[0x00, 0x2C, 0x01, 0xA7, 0x51, 0x01, 0xA5, 0x52, 0x01, 0xF7, 0x12]);
        terrain::decode_chunk(vec2(16, 0), foreground.blocks_mut(), &[
            0x01, 0x0B, 0x51, 0x01, 0x1A, 0x41, 0x01, 0x29, 0x31,
            0x01, 0x38, 0x21, 0x01, 0x65, 0x52,
            0x01, 0xAA, 0x23, 0x01, 0xAB, 0x23,
        ]);
        terrain::decode_chunk(vec2(2 * 16, 0), foreground.blocks_mut(), &[
            0x03, 0x05, 0xA1, 0x01, 0x54, 0x25,
            0x00, 0x6E,
            0x05, 0xAC, 0x01, 0x07, 0xAE, 0x20, 0x07, 0xAD, 0x00,
            0x06, 0x0C, 0x02, 0x07, 0x0E, 0x00,
            0x08, 0x7C, 0x22, 0x00, 0x46, 0x52, 0x46, 0x56, 0x62, 0x56, 0x62, 0x62,
        ]);
        terrain::decode_chunk(vec2(3 * 16, 0), foreground.blocks_mut(), &[
            0x05, 0x1E, 0x35, 0x05, 0xE8, 0x25,
            0x09, 0xD8, 0x08,
        ]);
        terrain::decode_chunk(vec2(4 * 16, 0), foreground.blocks_mut(), &[
            0x0A, 0xBA, 0x12,
        ]);
        terrain::decode_chunk(vec2(5 * 16, 0), foreground.blocks_mut(), &[
            0x06, 0x18, 0x25,
            0x04, 0x09, 0x75,
            0x07, 0x1E, 0x40,
            0x04, 0x5D, 0x31,
            0x01, 0x07, 0x11,
            0x08, 0x7D, 0x20, 0x49, 0x59, 0x49,
            0x0A, 0xBA, 0x32,
            /*0x08, 0xFA, 0x43,
            0x00, 0x00, 0x00, 0x5D, 0x5F,
            0x5D, 0x5E, 0x5F, 0x6D, 0x6F,
            0x6D, 0x6E, 0x6F, 0x6D, 0x6F,
            0x6D, 0x6E, 0x6F, 0x6D, 0x6F*/
        ]);
        terrain::decode_chunk(vec2(6 * 16, 0), foreground.blocks_mut(), &[
            0x02, 0xF0, 0xF1,
            /*0x08, 0xFA, 0x43,
            0x00, 0x00, 0x00, 0x5D, 0x5F,
            0x5D, 0x5E, 0x5F, 0x6D, 0x6F,
            0x6D, 0x6E, 0x6F, 0x6D, 0x6F,
            0x6D, 0x6E, 0x6F, 0x6D, 0x6F*/
        ]);
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
