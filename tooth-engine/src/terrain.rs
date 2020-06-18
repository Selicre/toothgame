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
                            0x52
                        } else {
                            0x62
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
            },
            2 => {  // column of blocks
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let block = params & 0x0F;
                let height = params >> 4;
                for y in y..y+height {
                    buf[x as usize + y as usize*256 + offset] = block;
                }
            },
            3 => {  // rectangle of land
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let height = params & 0x0F;
                let width = params >> 4;
                for rx in 0..=width {
                    for ry in 0..=height {
                        let block_low = if width == 0 {
                            0
                        } else if rx == 0 {
                            1
                        } else if rx == width {
                            3
                        } else {
                            2
                        };
                        let block_high = if height == 0 {
                            0
                        } else if ry == 0 {
                            1
                        } else if ry == height {
                            3
                        } else {
                            2
                        } + 4;
                        let block = (block_high << 4) | block_low;
                        buf[x as usize + rx as usize + (y as usize + ry as usize)*256 + offset] = block;
                    }
                }
            }
            _ => unreachable!()
        }
    }
}
