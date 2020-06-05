
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

