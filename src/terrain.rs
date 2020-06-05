use crate::vec2::Vec2;
use core::slice::Iter;

type Func = for<'a, 'b> fn(&mut Iter<'a, u8>, &'b mut [u8]);

pub fn decode_chunk(pos: Vec2<usize>, buf: &mut [u8], src: &[u8]) {
    let mut i = src.iter().cloned();
    let offset = pos.x + pos.y * 256;
    while let Some(id) = i.next() {
        match id {
            0 => {  // Horizontal stretch of land
                let params = i.next().unwrap();
                let height = params & 0x0F;
                let width = params >> 4;
                for x in 0..width*16 {
                    for y in height..16 {
                        buf[x as usize + y as usize*256 + offset] = if y == height {
                            0x22
                        } else {
                            0x32
                        };
                    }
                }
            },
            1 => {  // row of blocks
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let block = params & 0x0F;
                let width = params >> 4;
                for x in x..x+width {
                    buf[x as usize + y as usize*256 + offset] = block;
                }
            }
            _ => unreachable!()
        }
    }
}
