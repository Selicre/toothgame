use std::path::Path;

use std::collections::HashMap;
use image::GenericImageView;
use std::io;
use std::io::Write;
use std::fs::File;
use std::env;

#[derive(Debug)]
struct DataDef {
    offset: usize,
    end: usize,
    pal: usize,
}
impl DataDef {
    fn write(&self, mut into: impl io::Write, name: &str) {
        writeln!(into, "pub const {}: DataDef = {:?};", name, self).unwrap();
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gfx.rs");
    let bin_path = Path::new(&out_dir).join("gfx.bin");
    let mut f = File::create(&dest_path).unwrap();

    let mut data = vec![];
    let mut pal = vec![];

    let dune_bg = image::open("assets/bg/dune.png").unwrap().into_rgba();
    embed_bg(&dune_bg, &mut data, &mut pal).write(&mut f, "DUNE_BG");
    let dune_fg = image::open("assets/fg/dune.png").unwrap().into_rgba();
    embed_fg(&dune_fg, &mut data, &mut pal).write(&mut f, "DUNE_FG");
    let img = image::open("assets/sprites/toothpaste.png").unwrap().into_rgba();
    embed_fg(&img, &mut data, &mut pal).write(&mut f, "TOOTHPASTE");
    let img = image::open("assets/sprites/misc.png").unwrap().into_rgba();
    embed_fg(&img, &mut data, &mut pal).write(&mut f, "MISC");

    let comp = lz4::block::compress(&data, lz4::block::CompressionMode::HIGHCOMPRESSION(12).into(), false).unwrap();

    std::fs::write(bin_path, &comp).unwrap();
    std::fs::write("test.bin", &comp).unwrap();

    writeln!(f, r#"pub static mut GFX_DATA: [u8; {0:}] = [0; {0:}];"#, data.len()).unwrap();
    writeln!(f, r#"pub static GFX_DATA_LZ4: [u8; {}] = *include_bytes!(concat!(env!("OUT_DIR"), "/gfx.bin"));"#, comp.len()).unwrap();
    writeln!(f, "pub static PAL_DATA: [u32; {}] = {:?};", pal.len(), pal).unwrap();

    //panic!();

    //eprintln!("{}", data.len());
    //eprintln!("{:X?}", pal);
}

fn embed_bg(image: &image::RgbaImage, data: &mut Vec<u8>, pal: &mut Vec<u32>) -> DataDef {
    let mut palette = HashMap::new();
    let offset = data.len();
    let pal_offset = pal.len();
    data.extend(image.pixels().map(|c| {
        let c = if c.0[3] == 0 { image::Rgba([0; 4]) } else { *c };
        let len = palette.len();
        let id = palette.entry(c).or_insert_with(|| {
            pal.push(u32::from_le_bytes(c.0));
            len
        });

        *id as u8
    }));
    DataDef {
        offset,
        pal: pal_offset,
        end: data.len()
    }
}

fn embed_fg(image: &image::RgbaImage, data: &mut Vec<u8>, pal: &mut Vec<u32>) -> DataDef {
    let mut palette = HashMap::new();
    let offset = data.len();
    let pal_offset = pal.len();
    for ty in 0..image.height()/16 {
        for tx in 0..image.width()/16 {
            data.extend(image.view(tx*16, ty*16, 16, 16).pixels().map(|(_,_,c)| {
                let c = if c.0[3] == 0 { image::Rgba([0; 4]) } else { c };
                let len = palette.len();
                let id = palette.entry(c).or_insert_with(|| {
                    pal.push(u32::from_le_bytes(c.0));
                    len
                });
                *id as u8
            }));
        }
    }
    DataDef {
        offset,
        pal: pal_offset,
        end: data.len()
    }
}
