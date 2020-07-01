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
    pub fn block_at_mut(&mut self, mut at: Vec2<i32>) -> &mut u8 {
        at.x = at.x.max(0).min(256);
        at.y = at.y.max(0).min(256);
        &mut self.blocks[at.y as usize * 256 + at.x as usize]
    }
    pub fn block_at(&self, mut at: Vec2<i32>) -> u8 {
        at.x = at.x.max(0).min(256);
        at.y = at.y.max(0).min(256);
        self.blocks[at.y as usize * 256 + at.x as usize]
    }
    pub fn solidity_at(&self, at: Vec2<i32>) -> Solidity {
        use Solidity::*;
        let c = self.block_at(at);
        match c {
            0x00 => NonSolid,
            0x02 => Coin,
            0x04 => Semisolid,
            0x05 => HurtTop,
            0x49 => Slab,
            0x52 => Semisolid,
            0x46 => SlopeSteep(false),
            0x47 => SlopeSteep(true),
            0x56 => SlopeAssist { direction: false, steep: true  },
            0x57 => SlopeAssist { direction: true,  steep: true  },
            0x66 => SlopeAssist { direction: false, steep: false },
            0x74 => SlopeAssist { direction: true,  steep: false },
            0x64 => SlopeLow(false),
            0x65 => SlopeHigh(false),
            0x75 => SlopeHigh(true),
            0x76 => SlopeLow(true),
            0x5D ..= 0x5F => Semisolid,
            0x6D ..= 0x6F => NonSolid,
            _ => Solid
        }
        //if c == 0 { NonSolid } else { Solid }
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        use crate::graphics;
        let pal = graphics::DUNE_FG.get_pal();

        // TODO: copy rows of 16 pixels at a time?
        for (pos,i) in into.pixels() {
            let pos = pos + camera;
            let block_pos = pos.map(|c| c >> 4);
            let block_offset = pos & 15;
            let offset_addr = (block_offset.x + block_offset.y * 16) as usize;
            let block_addr = (block_pos.x + block_pos.y * 256) as usize;
            let block_id = self.blocks[block_addr];
            //let block_id = self.block_at(block_pos);

            let px = pal[self.fg_block(block_id as usize)[offset_addr] as usize];
            if px != 0 { *i = px; }
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum Solidity {
    NonSolid,
    Solid,
    Coin,
    Semisolid,
    HurtTop,
    Slab,
    SlopeHigh(bool),
    SlopeLow(bool),
    SlopeSteep(bool),
    SlopeAssist {
        direction: bool,
        steep: bool
    },
}

