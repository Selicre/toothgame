use crate::framebuffer::Framebuffer;
use crate::vec2::Vec2;
use crate::graphics::{self, DataDef};


static mut BLOCKS: [u8; 65536] = [0; 65536];

pub struct Foreground {
    blocks: &'static mut [u8; 65536],
    gfx: DataDef
}

impl Foreground {
    pub fn new() -> Foreground {
        Foreground {
            blocks: unsafe { &mut BLOCKS },
            gfx: graphics::DUNE_FG
        }
    }
    fn fg_block(&self, tile: usize) -> &[u8] {
        let data = self.gfx.get_data();
        &data[tile * 256 .. tile * 256 + 256]
    }
    pub fn blocks_mut(&mut self) -> &mut [u8] {
        self.blocks
    }
    pub fn block_at_mut(&mut self, mut at: Vec2<i32>) -> &mut u8 {
        at.x = at.x.max(0).min(255);
        at.y = at.y.max(0).min(255);
        &mut self.blocks[(at.y * 256 + at.x) as usize]
    }
    pub fn block_at(&self, mut at: Vec2<i32>) -> u8 {
        at.x = at.x.max(0).min(255);
        at.y = at.y.max(0).min(255);
        self.blocks[(at.y * 256 + at.x) as usize]
    }
    pub fn solidity_at(&self, at: Vec2<i32>) -> Solidity {
        use Solidity::*;
        let c = self.block_at(at);
        match c {
            0x00 => NonSolid,
            0x02 => Coin,
            0x04 => Semisolid,
            0x05 => HurtTop,
            0x06 => Semisolid,
            0x49 => Slab,
            0x52 => EjectUp,
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
            0x80 ..= 0xFF => NonSolid,
            _ => Solid
        }
        //if c == 0 { NonSolid } else { Solid }
    }
    pub fn sample_pixel(&self, pos: Vec2<i32>) -> Option<u32> {
        use crate::graphics;
        let pal = graphics::DUNE_FG.get_pal();

        let block_pos = pos.map(|c| c >> 4);
        let block_offset = pos & 15;
        let offset_addr = (block_offset.x + block_offset.y * 16) as usize;
        let block_addr = (block_pos.x + block_pos.y * 256) as usize;
        let block_id = *self.blocks.get(block_addr)?;
        //let block_id = self.block_at(block_pos);

        pal.get(*self.fg_block(block_id as usize).get(offset_addr)? as usize).copied()
    }
    pub fn render(&self, camera: Vec2<i32>, into: &mut Framebuffer) {
        // TODO: copy rows of 16 pixels at a time?
        for (pos,i) in into.pixels() {
            let pos = pos + camera;
            let px = self.sample_pixel(pos).unwrap();
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
    EjectUp,
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

