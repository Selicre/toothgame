use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};

pub struct Background {
    gfx_id: usize
}

impl Background {
    pub const fn new() -> Background {
        Background {
            gfx_id: 0
        }
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        use crate::graphics;
        let data = graphics::DUNE_BG.get_data();
        let pal = graphics::DUNE_BG.get_pal();
        for (pos,i) in into.pixels() {
            let mut pos = pos + camera;
            if pos.y >= 64 {
                pos.x += camera.x * ((pos.y - 58) / 6) / 3;
            }
            pos.x %= 320;
            let tile_pos = pos / 8;
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
