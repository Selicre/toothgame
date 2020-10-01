use crate::vec2::{vec2, Vec2};
use core::slice::Iter;
use crate::rand::RandState;

pub fn decode_area(buf: &mut [u8], mut src: &[u8]) {
    let len = ((src[0] as usize) << 8) + src[1] as usize;
    src = &src[2..];
    decode_land(buf, &src[..len]);
    src = &src[len..];
    while src.len() != 0 {
        let old = src;
        decode_object(buf, &mut src);
    }
}

pub fn decode_land_chunk(src: &mut &[u8]) -> [u8; 5] {
    let mut buf = [0; 5];
    buf.copy_from_slice(&src[..5]);
    *src = &src[5..];
    buf
}


pub fn decode_land(buf: &mut [u8], mut src: &[u8]) {
    decode_land_with(buf, core::iter::from_fn(|| {
        if src.len() != 0 {
            Some(decode_land_chunk(&mut src))
        } else {
            None
        }
    }));
}

pub fn decode_land_with(buf: &mut [u8], mut src: impl Iterator<Item=[u8;5]>) {
    let mut rand = RandState::new(0);
    //while src.len() != 0 {
    //    let [x,y,w,h,b] = decode_land_chunk(&mut src);
    while let Some([x,y,w,h,b]) = src.next() {
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
                if b == 0x52 {  // top soil
                    let ty = rand.next() & 0x0F;
                    if ty < 4 {
                        buf[(x as usize) + (y as usize - 1) * 256] = 0x80 + ty as u8;
                    }
                }
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

pub fn decode_object(buf: &mut [u8], i: &mut &[u8]) {
    let id = i[0];
    let y = i[1];
    let x = i[2];
    let params = i[3];
    let b = params & 0x0F;
    let a = params >> 4;
    *i = &i[4..];
    match id {
        1 => {  // row of blocks
            let block = b;
            let width = a;
            for x in x..x+width {
                buf[x as usize + y as usize*256] = block;
            }
        },
        2 => {  // column of blocks
            let block = b;
            let height = a;
            for y in y..y+height {
                buf[x as usize + y as usize*256] = block;
            }
        },
        5 => {  // land gentle slope
            let height = b;
            let is_up = (a) & 0x01 != 0;
            let is_filled = (a >> 1) & 0x01 != 0;
            if is_up {
                buf[x as usize + (y as usize) * 256] = 0x66;
                for i in 0..height {
                    for j in 0..3 {
                        buf[x as usize + (i * 2 + j) as usize + (y as usize - i as usize - 1) * 256] = 0x64 + j;
                    }
                    if is_filled {
                        for j in i*2..height*2+1 {
                            buf[x as usize + (j + 1) as usize + (y as usize - i as usize) * 256] = 0x62;
                        }
                    }
                }
                if is_filled {
                    buf[x as usize + (height as usize * 2 + 1) + (y as usize - height as usize) * 256] = 0x62;
                }
                buf[x as usize + 0 + (height as usize * 2) + (y as usize - height as usize - 1) * 256] = 0x64;
                buf[x as usize + 1 + (height as usize * 2) + (y as usize - height as usize - 1) * 256] = 0x65;
            } else {
                buf[x as usize + (y as usize) * 256] = 0x75;
                buf[x as usize + 1 + (y as usize) * 256] = 0x76;
                for i in 0..height {
                    for j in 0..3 {
                        buf[x as usize + (i * 2 + j + 1) as usize + (y as usize + i as usize + 1) * 256] = 0x74 + j;
                    }
                    if is_filled {
                        for j in 0..i*2+3 {
                            buf[x as usize + (j) as usize + (y as usize + i as usize + 2) * 256] = 0x62;
                        }
                    }
                    if is_filled {
                        buf[x as usize + (y as usize + 1 as usize) * 256] = 0x62;
                    }
                }
                buf[x as usize + 1 + (height as usize * 2) + (y as usize + height as usize + 1) * 256] = 0x74;
            }
        },
        6 => {  // land steep slope
            let height = b;
            let is_up = (a) & 0x01 != 0;
            let is_filled = (a >> 1) & 0x01 != 0;
            for i in 0..height {
                for j in 0..2 {
                    if is_up {
                        buf[x as usize + (i) as usize + (y as usize - i as usize + j as usize) * 256] = 0x46 + j * 0x10;
                    } else {
                        buf[x as usize + (i) as usize + (y as usize + i as usize + j as usize) * 256] = 0x47 + j * 0x10;
                    }
                }
                if is_filled {
                    if is_up {
                        for j in 0..i {
                            buf[x as usize + (i) as usize + (y as usize - j as usize + 1) * 256] = 0x62;
                        }
                    } else {
                        for j in i+1..height {
                            buf[x as usize + (i) as usize + (y as usize + j as usize + 1) * 256] = 0x62;
                        }
                    }
                }
            }
        },
        8 => {  // uncompressed block sequence
            let height = b;
            let width = a;
            for ry in 0..=height {
                for rx in 0..=width {
                    let block = i[0];
                    *i = &i[1..];
                    buf[x as usize + rx as usize + (y as usize + ry as usize)*256] = block;
                }
            }
        },
        0x0A => {   // Semisolid
            let height = (b) + 1;   // min width = 2
            let width = (a) + 1;
            for ry in 0..=height {
                for rx in 0..=width {
                    let block = if ry == 0 { 0x50 } else { 0x60 } | match rx {
                        0 => 0x0D,
                        c if c == width => 0x0F,
                        _ => 0x0E
                    };
                    buf[x as usize + rx as usize + (y as usize + ry as usize)*256] = block;
                }
            }
        }
        _ => {}
    }
}
