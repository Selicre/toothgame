use std::path::Path;

use std::collections::{HashMap, HashSet};
use image::GenericImageView;
use std::io;
use std::fs::File;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gfx.rs");
    let mut f = File::create(&dest_path).unwrap();
    let dune_bg = image::open("assets/bg/dune.png").unwrap().into_rgba();
    compress_bg(&mut f, "DUNE_BG", dune_bg);
    let dune_fg = image::open("assets/fg/dune.png").unwrap().into_rgba();
    compress_fg(&mut f, "DUNE_FG", dune_fg);
}

/*
fn embed(mut f: impl io::Write, name: &str, image: image::RgbaImage) {
    let mut palette = HashMap::new();
    let image = image.pixels().map(|c| {
        let c = if c.0[3] == 0 { image::Rgba([0; 4]) } else { *c };
        let len = palette.len();
        let id = palette.entry(c).or_insert(len);
        *id
    }).collect::<Vec<_>>();
    writeln!(f, "static {}_PIXELS: [u8; {}] = [", name, image.len());
    for i in image.iter() {
        writeln!(f, "    {}, ", i);
    }
    writeln!(f, "];");
    writeln!(f, "static {}_PAL: [u32; {}] = [", name, palette.len());
    let mut palette = palette.iter().collect::<Vec<_>>();
    palette.sort_by_key(|c| c.1);
    for (i,_id) in palette.iter() {
        let [r,g,b,a] = i.0;
        let color = a as u32 * 0x1000000 + r as u32 * 0x10000 + g as u32 * 0x100 + b as u32;
        writeln!(f, "    0x{:08X}, ", color);
    }
    writeln!(f, "];");
}*/

fn compress_bg(mut f: impl io::Write, name: &str, image: image::RgbaImage) {
    let mut palette = HashMap::new();
    let mut tiles = HashMap::new();
    let mut map = vec![];

    for ty in 0..image.height()/8 {
        for tx in 0..image.width()/8 {
            let view = image.view(tx*8, ty*8, 8, 8);
            let tile = view.pixels().map(|(x,y,mut c)| {
                let len = palette.len();
                if c.0[3] == 0 { c = image::Rgba([0; 4]) };
                eprintln!("color {:X?} - entry {:X?}", c, palette.get(&c));
                let id = palette.entry(c).or_insert(len);
                *id
            }).collect::<Vec<_>>();
            let len = tiles.len();
            let id = tiles.entry(tile).or_insert(len);
            map.push(*id);
        }
    }

    eprintln!("palette: {:X?}", palette);
    writeln!(f, "static {}_TILES: [u8; {}] = [", name, tiles.len() * 64);
    let mut c = tiles.into_iter().map(|c| (c.1, c.0)).collect::<Vec<(usize,_)>>();
    c.sort();
    for (id,tile) in c {
        write!(f, "    ");
        for px in tile.iter() {
            write!(f, "{}, ", px);
        }
        writeln!(f);
    }
    writeln!(f, "];");

    writeln!(f, "static {}_MAP: [u8; {}] = [", name, map.len());
    for i in map {
        writeln!(f, "    {},", i);
    }
    writeln!(f, "];");
    writeln!(f, "static {}_WIDTH: i32 = {};", name, image.width() / 8);
    writeln!(f, "static {}_PAL: [u32; {}] = [", name, palette.len());
    let mut palette = palette.iter().collect::<Vec<_>>();
    palette.sort_by_key(|c| c.1);
    for (i,_id) in palette.iter() {
        let color = u32::from_le_bytes(i.0);
        writeln!(f, "    0x{:08X}, ", color);
    }
    writeln!(f, "];");
}


fn compress_fg(mut f: impl io::Write, name: &str, image: image::RgbaImage) {
    let mut palette = HashMap::new();
    let mut tiles = HashMap::new();
    let mut map = vec![];
    assert_eq!(image.width(), 256);     // For sanity
    assert_eq!(image.height(), 256);

    for ty in 0..image.height()/8 {
        for tx in 0..image.width()/8 {
            let view = image.view(tx*8, ty*8, 8, 8);
            let tile = view.pixels().map(|(x,y,mut c)| {
                let len = palette.len();
                if c.0[3] == 0 { c = image::Rgba([0; 4]) };
                eprintln!("color {:X?} - entry {:X?}", c, palette.get(&c));
                let id = palette.entry(c).or_insert(len);
                *id
            }).collect::<Vec<_>>();
            let len = tiles.len();
            let id = tiles.entry(tile).or_insert(len);
            map.push(*id);
        }
    }

    eprintln!("palette: {:X?}", palette);
    writeln!(f, "static {}_TILES: [u8; {}] = [", name, tiles.len() * 64);
    let mut c = tiles.into_iter().map(|c| (c.1, c.0)).collect::<Vec<(usize,_)>>();
    c.sort();
    for (id,tile) in c {
        write!(f, "    ");
        for px in tile.iter() {
            write!(f, "{}, ", px);
        }
        writeln!(f);
    }
    writeln!(f, "];");

    writeln!(f, "static {}_MAP: [u8; {}] = [", name, map.len());
    for y in 0..16 {
        for x in 0..16 {
            writeln!(f, "    {}, {}, {}, {},",
                     map[x*2 + y * 64],
                     map[x*2 + y * 64 + 1],
                     map[x*2 + y * 64 + 32],
                     map[x*2 + y * 64 + 33]);
        }
    }
    writeln!(f, "];");
    writeln!(f, "static {}_PAL: [u32; {}] = [", name, palette.len());
    let mut palette = palette.iter().collect::<Vec<_>>();
    palette.sort_by_key(|c| c.1);
    for (i,_id) in palette.iter() {
        let color = u32::from_le_bytes(i.0);
        writeln!(f, "    0x{:08X}, ", color);
    }
    writeln!(f, "];");
}
