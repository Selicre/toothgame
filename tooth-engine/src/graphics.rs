use crate::vec2::{Vec2, vec2};

pub struct DataDef {
    pub offset: usize,
    pub end: usize,
    pub pal: usize,
}

impl DataDef {
    pub fn get_data(&self) -> &'static [u8] {
        &GFX_DATA[self.offset..self.end]
    }
    pub fn get_pal(&self) -> &'static [u32] {
        &PAL_DATA[self.pal..]
    }
}

include!(concat!(env!("OUT_DIR"), "/gfx.rs"));
