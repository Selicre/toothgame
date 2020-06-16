use crate::vec2::{Vec2, vec2};
use crate::lz4;

pub struct DataDef {
    pub offset: usize,
    pub end: usize,
    pub pal: usize,
}

impl DataDef {
    pub fn get_data(&self) -> &'static [u8] {
        unsafe { &GFX_DATA[self.offset..self.end] }
    }
    pub fn get_pal(&self) -> &'static [u32] {
        &PAL_DATA[self.pal..]
    }
}

pub fn init() {
    unsafe { lz4::decompress(&GFX_DATA_LZ4, &mut GFX_DATA) };
}

include!(concat!(env!("OUT_DIR"), "/gfx.rs"));
