use crate::lz4;
use crate::vec2::{vec2, Vec2};
use crate::framebuffer::Framebuffer;

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

pub fn draw_text(fb: &mut Framebuffer, position: &mut Vec2<i32>, msg: &[u8]) {
    let data = BOLDFACE.get_data();
    for c in msg.iter() {
        if *c < 0x20 { continue; }
        let offset = (*c as usize - 0x20) * 64;
        for i in 0..64 {
            let pixel = data[offset + i];
            if pixel == 1 {
                *fb.pixel(*position + vec2(i as i32 % 8 + 1,i as i32 / 8 + 1)).unwrap() = 0xFF888888;
                *fb.pixel(*position + vec2(i as i32 % 8,i as i32 / 8)).unwrap() = 0xFFFFFFFF;
            }
        }
        *position = *position + vec2(8,0);
        if position.x + 8 > 320 { position.x = 0; position.y += 8; }
    }
}

include!(concat!(env!("OUT_DIR"), "/gfx.rs"));
