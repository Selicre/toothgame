use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};
use crate::graphics::{self, DataDef};

pub struct Background {
    gfx: DataDef
}

impl Background {
    pub const fn new() -> Background {
        Background {
            gfx: graphics::DUNE_BG
        }
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        let data = self.gfx.get_data();
        let pal = self.gfx.get_pal();
        let camera = vec2(camera.x / 4, -52);
        for (pos,i) in into.pixels() {
            let mut pos = pos + camera;
            if pos.y >= 64 {
                pos.x += camera.x * ((pos.y - 58) / 6) / 6;
            }
            pos.x %= 320;
            if pos.y < 0 {
                *i = 0xff_b5_9f_e6;
                continue;
            }
            let px = pal[data[(pos.y * 320 + pos.x) as usize] as usize];
            if px != 0 {
                *i = px;
            } else {
                *i = 0xff_b5_9f_e6;
            }
        }
    }
}
