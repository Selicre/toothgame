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
        //entity_set.spawn(entity::star(vec2(1480, 60) * 256));
        entity_set.spawn(entity::key(vec2(0x288, 0x23F) * 256));
        entity_set.spawn(entity::lock(vec2(1640, 0x2CE) * 256));
        entity_set.spawn(entity::sign(vec2(472, 0x2BF) * 256, b"THIS IS REALLY LARGE TEXT,YOU\nHAVE TO WRAP IT"));
        entity_set.spawn(entity::sign(vec2(0x2F7, 0x2DF) * 256, b"THIS IS GIANT TEXT.\nBIG.\nTREMENDOUS.\nFANTASTIC.\nMANY LINES."));
        //entity_set.spawn(entity::tomato(vec2(400, 60) * 256));
        entity_set.player.set_pos(vec2(0x68, 0x2BF) * 256);
        LevelState {
            camera: vec2(0,60),
            data: LevelData {
                level_size: vec2(4096, 2048),
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


pub fn decomp_level(foreground: &mut Foreground) {
    terrain::decode_area(foreground.blocks_mut(), include_bytes!("../../../../level_demo.bin"));
}

