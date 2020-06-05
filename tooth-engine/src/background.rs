use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};

static mut BG_BUF: [u32; 65536] = [0; 65536];

fn bg_buf() -> &'static mut [u32] {
    unsafe { &mut BG_BUF }
}

pub struct Background {
    map: [u8; 1024],
    size: Vec2<i32>
}
impl Background {
    pub const fn new() -> Background {
        Background {
            map: [0; 1024],
            size: vec2(0, 0),
        }
    }
    pub fn unpack_gfx(&mut self, tileset: usize) {
        use crate::graphics;
        self.size = graphics::decompress_bg(tileset, bg_buf(), &mut self.map);
    }
    fn bg_tile(&self, tile: usize) -> &[u32] {
        &bg_buf()[tile * 64 .. tile * 64 + 64]
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        // TODO: copy rows of 8 pixels at a time?
        for (pos,i) in into.pixels() {
            let pos = pos + camera;
            let tile_pos = pos / 8;
            if pos.y < 0 || pos.y > self.size.y*8 {
                *i = 0xe69fb5ff;
                continue;
            }
            let tile_pos = vec2(tile_pos.x % self.size.x, tile_pos.y);
            let tile_offset = pos & 7;
            let offset_addr = (tile_offset.x + tile_offset.y * 8) as usize;
            let tile_addr = (tile_pos.x + tile_pos.y * self.size.x) as usize;
            let tile_addr = tile_addr & 1023;
            let tile_id = self.map[tile_addr];
            let px = self.bg_tile(tile_id as usize)[offset_addr];
            if px != 0 {
                *i = px;
            } else {
                *i = 0xe69fb5ff;
            }
        }
    }
}
