use crate::lz4;
use crate::vec2::{vec2, Vec2};
use crate::framebuffer::Surface;

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

pub fn draw_text<S: Surface>(fb: &mut S, position: &mut Vec2<i32>, msg: &[u8]) {
    let data = BOLDFACE.get_data();
    let init_x = position.x;
    for c in msg.iter() {
        if *c == b'\n' { position.x = init_x; position.y += 8; }
        if *c < 0x20 { continue; }
        let offset = (*c as usize - 0x20) * 64;
        for i in 0..64 {
            let pixel = data[offset + i];
            if pixel == 1 {
                fb.pixel(*position + vec2(i as i32 % 8 + 1,i as i32 / 8 + 1)).map(|c| *c = 0xFF888888);
                fb.pixel(*position + vec2(i as i32 % 8,i as i32 / 8)).map(|c| *c = 0xFFFFFFFF);
            }
        }
        *position = *position + vec2(8,0);
    }
}

include!(concat!(env!("OUT_DIR"), "/gfx.rs"));
