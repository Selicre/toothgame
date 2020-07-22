use crate::vec2::Vec2;
use core::slice::Iter;

pub fn decode_land(buf: &mut [u8], src: &[u8]) {
    let mut iter = src.iter();
    while iter.as_slice().len() != 0 {
        let x = *iter.next().unwrap();
        let y = *iter.next().unwrap();
        let params = *iter.next().unwrap();
        let mut h = params & 0x0F;
        let mut w = params >> 4;
        if w == 0 { w = *iter.next().unwrap(); }
        if h == 0 { h = *iter.next().unwrap(); }
        let b = *iter.next().unwrap();
        for i in x..x+w {
            for j in y..y+h {
                buf[i as usize + j as usize * 256] = b;
            }
        }
    }

    for y in 0u8..=255 {
        for x in 0u8..=255 {
            let top =    buf[x as usize + y.saturating_sub(1) as usize * 256] != 0;
            let bottom = buf[x as usize + y.saturating_add(1) as usize * 256] != 0;
            let left =   buf[x.saturating_sub(1) as usize + y as usize * 256] != 0;
            let right =  buf[x.saturating_add(1) as usize + y as usize * 256] != 0;

            let this = &mut buf[x as usize + y as usize * 256];

            if *this == 0x62 {
                let bh = [0, 1, 3, 2][((top as usize) << 1) + bottom as usize] + 4;
                let bl = [0, 1, 3, 2][((left as usize) << 1) + right as usize];
                let b = (bh << 4) + bl;
                *this = b;
            } else if *this == 0x00 {
                if top && left {
                    buf[(x as usize - 1) + (y as usize - 1) * 256] = 0x44;
                }
                if top && right {
                    buf[(x as usize + 1) + (y as usize - 1) * 256] = 0x45;
                }
                if bottom && left {
                    buf[(x as usize - 1) + (y as usize + 1) * 256] = 0x54;
                }
                if bottom && right {
                    buf[(x as usize + 1) + (y as usize + 1) * 256] = 0x55;
                }
            }
        }
    }
}

pub fn decode_chunk(pos: Vec2<usize>, buf: &mut [u8], src: &[u8]) {
    let mut i = src.iter().cloned();
    let offset = pos.x + pos.y * 256;
    while let Some(id) = i.next() {
        match id {
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
            5 => {  // land gentle slope
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let height = params & 0x0F;
                let is_up = (params >> 4) & 0x01 != 0;
                let is_filled = (params >> 5) & 0x01 != 0;
                if is_up {
                    buf[x as usize + (y as usize) * 256 + offset] = 0x66;
                    for i in 0..height {
                        for j in 0..3 {
                            buf[x as usize + (i * 2 + j) as usize + (y as usize - i as usize - 1) * 256 + offset] = 0x64 + j;
                        }
                        if is_filled {
                            for j in i*2..height*2+1 {
                                buf[x as usize + (j + 1) as usize + (y as usize - i as usize) * 256 + offset] = 0x62;
                            }
                        }
                    }
                    if is_filled {
                        buf[x as usize + (height as usize * 2 + 1) + (y as usize - height as usize) * 256 + offset] = 0x62;
                    }
                    buf[x as usize + 0 + (height as usize * 2) + (y as usize - height as usize - 1) * 256 + offset] = 0x64;
                    buf[x as usize + 1 + (height as usize * 2) + (y as usize - height as usize - 1) * 256 + offset] = 0x65;
                } else {
                    buf[x as usize + (y as usize) * 256 + offset] = 0x75;
                    buf[x as usize + 1 + (y as usize) * 256 + offset] = 0x76;
                    for i in 0..height {
                        for j in 0..3 {
                            buf[x as usize + (i * 2 + j + 1) as usize + (y as usize + i as usize + 1) * 256 + offset] = 0x74 + j;
                        }
                        if is_filled {
                            for j in 0..i*2+3 {
                                buf[x as usize + (j) as usize + (y as usize + i as usize + 2) * 256 + offset] = 0x62;
                            }
                        }
                        if is_filled {
                            buf[x as usize + (y as usize + 1 as usize) * 256 + offset] = 0x62;
                        }
                    }
                    buf[x as usize + 1 + (height as usize * 2) + (y as usize + height as usize + 1) * 256 + offset] = 0x74;
                }
            },
            6 => {  // land steep slope
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let height = params & 0x0F;
                let is_up = (params >> 4) & 0x01 != 0;
                let is_filled = (params >> 5) & 0x01 != 0;
                for i in 0..height {
                    for j in 0..2 {
                        if is_up {
                            buf[x as usize + (i) as usize + (y as usize - i as usize + j as usize) * 256 + offset] = 0x46 + j * 0x10;
                        } else {
                            buf[x as usize + (i) as usize + (y as usize + i as usize + j as usize) * 256 + offset] = 0x47 + j * 0x10;
                        }
                    }
                    if is_filled {
                        if is_up {
                            for j in 0..i {
                                buf[x as usize + (i) as usize + (y as usize - j as usize + 1) * 256 + offset] = 0x62;
                            }
                        } else {
                            for j in i+1..height {
                                buf[x as usize + (i) as usize + (y as usize + j as usize + 1) * 256 + offset] = 0x62;
                            }
                        }
                    }
                }
            },
            8 => {  // uncompressed block sequence
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let height = params & 0x0F;
                let width = params >> 4;
                for ry in 0..=height {
                    for rx in 0..=width {
                        let block = i.next().unwrap();
                        buf[x as usize + rx as usize + (y as usize + ry as usize)*256 + offset] = block;
                    }
                }
            },
            0x0A => {   // Semisolid
                let params = i.next().unwrap();
                let y = params & 0x0F;
                let x = params >> 4;
                let params = i.next().unwrap();
                let height = (params & 0x0F) + 1;   // min width = 2
                let width = (params >> 4) + 1;
                for ry in 0..=height {
                    for rx in 0..=width {
                        let block = if ry == 0 { 0x50 } else { 0x60 } | match rx {
                            0 => 0x0D,
                            c if c == width => 0x0F,
                            _ => 0x0E
                        };
                        buf[x as usize + rx as usize + (y as usize + ry as usize)*256 + offset] = block;
                    }
                }

            }
            _ => {}
        }
    }
}
