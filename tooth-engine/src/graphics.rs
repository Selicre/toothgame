use crate::vec2::{Vec2, vec2};

include!(concat!(env!("OUT_DIR"), "/gfx.rs"));

pub fn decompress_fg(idx: usize, into: &mut [u32]) {
    let (tiles, map, pal) = match idx {
        0 => (&DUNE_FG_TILES, &DUNE_FG_MAP, &DUNE_FG_PAL),
        _ => panic!()
    };
    for (i, idx) in map.chunks(4).enumerate() {
        let tile_buf = &mut into[i*256..i*256+256];
        for (i, px) in tile_buf.iter_mut().enumerate() {
            let x = i % 16;
            let y = i / 16;
            let id = (x >= 8) as usize + 2 * (y >= 8) as usize;
            let tx = x % 8;
            let ty = y % 8;
            let tile_offset = tx + ty * 8;
            *px = pal[tiles[idx[id] as usize * 64 + tile_offset] as usize];
        }
    }
}



pub fn decompress_bg(idx: usize, gfx_buf: &mut [u32], map_buf: &mut [u8]) -> Vec2<i32> {
    let (tiles, map, pal, width) = match idx {
        0 => (&DUNE_BG_TILES, &DUNE_BG_MAP, &DUNE_BG_PAL, DUNE_BG_WIDTH),
        _ => panic!()
    };
    for (i, idx) in tiles.iter().enumerate() {
        gfx_buf[i] = pal[*idx as usize];
    }
    map_buf[..map.len()].copy_from_slice(map);
    vec2(width, map.len() as i32 / width)
}
