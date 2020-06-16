use crate::framebuffer::Framebuffer;
use crate::vec2::Vec2;

pub struct Foreground {
    blocks: [u8; 65536],
    gfx_id: usize
}

impl Foreground {
    pub const fn new() -> Foreground {
        Foreground {
            blocks: [0; 65536],
            gfx_id: 0
        }
    }
    fn fg_block(&self, tile: usize) -> &[u8] {
        use crate::graphics;
        let data = graphics::DUNE_FG.get_data();
        &data[tile * 256 .. tile * 256 + 256]
    }
    pub fn blocks_mut(&mut self) -> &mut [u8] {
        &mut self.blocks
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        use crate::graphics;
        let pal = graphics::DUNE_FG.get_pal();

        // TODO: copy rows of 16 pixels at a time?
        for (pos,i) in into.pixels() {
            let pos = pos + camera;
            let block_pos = pos / 16;
            let block_offset = pos & 15;
            let offset_addr = (block_offset.x + block_offset.y * 16) as usize;
            let block_addr = (block_pos.x + block_pos.y * 256) as usize;
            let block_id = self.blocks[block_addr];

            let px = pal[self.fg_block(block_id as usize)[offset_addr] as usize];
            if px != 0 { *i = px; }
        }
    }
}
