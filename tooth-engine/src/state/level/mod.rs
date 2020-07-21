use super::GameState;

use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::vec2::{Vec2, vec2};
use crate::foreground::Foreground;
use crate::background::Background;
use crate::entity::{self, EntitySet};
use crate::terrain;
use crate::graphics;

pub mod hud;

pub struct LevelState {
    pub camera: Vec2<i32>,
    pub foreground: Foreground,
    pub background: Background,
    pub entity_set: EntitySet,
    pub data: LevelData,
    pub hud: hud::Hud,
    pub buttons: Buttons
}

pub struct LevelData {
    pub level_size: Vec2<i32>,
    pub fadein_timer: i32,
    pub coins: i32,
    pub score: i32,
    pub timer: Option<i32>
}

impl LevelState {
    pub fn new() -> Self {
        use crate::graphics;
        let background = Background::new();
        let mut foreground = Foreground::new();
        decomp_level(&mut foreground);
        let mut entity_set = EntitySet::new();
        entity_set.spawn(entity::star(vec2(1480, 60) * 256));
        entity_set.spawn(entity::key(vec2(1024, 96) * 256));
        entity_set.spawn(entity::lock(vec2(1640, 206) * 256));
        entity_set.spawn(entity::sign(vec2(472, 190) * 256, b"THIS IS REALLY LARGE TEXT,YOU\nHAVE TO WRAP IT"));
        entity_set.spawn(entity::sign(vec2(1024, 120) * 256, b"THIS IS GIANT TEXT.\nBIG.\nTREMENDOUS.\nFANTASTIC.\nMANY LINES."));
        entity_set.spawn(entity::tomato(vec2(400, 60) * 256));
        entity_set.player.set_pos(vec2(472, 60) * 256);
        LevelState {
            camera: vec2(0,60),
            data: LevelData {
                level_size: vec2(2048, 256),
                fadein_timer: 0,
                coins: 0,
                score: 0,
                timer: Some(0)
            },
            foreground,
            background,
            entity_set,
            hud: hud::Hud::new(),
            buttons: Buttons::new()
        }
    }
    pub fn run(&mut self, fb: &mut Framebuffer, buttons: Buttons) -> Option<GameState> {
        self.buttons = buttons;
        let self_ptr = self as *mut _;
        self.entity_set.run(self_ptr);

        let camera_target = self.entity_set.player.pos() / 256 - Framebuffer::size() / 2 + vec2(0, 16);

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

        self.camera = self.camera.zip(
            self.data.level_size - Framebuffer::size(),
            |c,m| c.max(0).min(m)
        );

        self.background.render(self.camera, fb);
        self.foreground.render(self.camera, fb);
        self.entity_set.render(self.camera, fb);

        self.hud.render(fb, self_ptr);


        /*let mut target = *b"POS 00000 00000";

        let pos = self.entity_set.player.pos();
        hex_format(pos.x/16, &mut target[4..9], 5);
        hex_format(pos.y/16, &mut target[10..15], 5);

        graphics::draw_text(fb, &mut vec2(8, 8), &target[..]);*/

        if self.data.fadein_timer < Framebuffer::size().x.max(Framebuffer::size().y) {
            self.data.fadein_timer += 8;
            let center = self.entity_set.player.pos() / 256 - self.camera - vec2(0, 24);
            for (pos,px) in fb.pixels() {
                let dist = pos - center;
                if dist.x*dist.x + dist.y*dist.y > self.data.fadein_timer*self.data.fadein_timer {
                    *px = 0xFF000000;
                }
            }
        }
        None
    }
}

fn hex_format(num: i32, target: &mut [u8], places: usize) {
    for i in 0..places {
        let ch = ((num >> ((places-1-i)*4)) & 0xF) as u8;
        target[i] = ch + if ch > 9 {
            b'A' - 10
        } else {
            b'0'
        };
    }
}


fn decomp_level(foreground: &mut Foreground) {
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
        0x02, 0x6B, 0x31,
        0x03, 0x63, 0x57,
    ]);
    terrain::decode_chunk(vec2(7 * 16, 0), foreground.blocks_mut(), &[
        0x02, 0xF0, 0xE1,
        /*0x08, 0xFA, 0x43,
        0x00, 0x00, 0x00, 0x5D, 0x5F,
        0x5D, 0x5E, 0x5F, 0x6D, 0x6F,
        0x6D, 0x6E, 0x6F, 0x6D, 0x6F,
        0x6D, 0x6E, 0x6F, 0x6D, 0x6F*/
    ]);
}

