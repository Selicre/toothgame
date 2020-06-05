use crate::framebuffer::Framebuffer;
use crate::vec2::Vec2;

struct ForegroundGfx {
    vram: [u32; 65536]
}
impl ForegroundGfx {
    const fn new() -> ForegroundGfx {
        ForegroundGfx {
            vram: [0; 65536]
        }
    }
    fn unpack(&mut self, tileset: usize) {
        use crate::graphics;
        graphics::decompress_fg(tileset, &mut self.vram);
    }
    fn fg_tile(&self, tile: usize) -> &[u32] {
        &self.vram[tile * 256 .. tile * 256 + 256]
    }
}

pub struct Foreground {
    blocks: [u8; 65536],
    gfx: ForegroundGfx
}

impl Foreground {
    pub const fn new() -> Foreground {
        Foreground {
            blocks: [0; 65536],
            gfx: ForegroundGfx::new()
        }
    }
    pub fn unpack_gfx(&mut self, id: usize) {
        self.gfx.unpack(id);
    }
    pub fn blocks_mut(&mut self) -> &mut [u8] {
        &mut self.blocks
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        use crate::graphics;
        // TODO: copy rows of 16 pixels at a time?
        for (pos,i) in into.pixels() {
            let pos = pos + camera;
            let block_pos = pos / 16;
            let block_offset = pos & 15;
            let offset_addr = (block_offset.x + block_offset.y * 16) as usize;
            let block_addr = (block_pos.x + block_pos.y * 256) as usize;
            let block_id = self.blocks[block_addr];
            *i = self.gfx.fg_tile(block_id as usize)[offset_addr];
        }
    }
}
