
use tooth_engine::*;

use framebuffer::{Surface as FbSurface};
use crate::vec2::{Vec2,vec2};

use glium::program;
use glium::Surface;
use glium::glutin::event::WindowEvent;
fn main() {
    graphics::init();
    let mut fb = vec![0u32; 960 * 540].into_boxed_slice();
    let mut event_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(1920.0, 1080.0))
        .with_title("Level editor");
    let cb = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_double_buffer(Some(true))
        .with_srgb(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        glium::implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
            ]
        ).unwrap()
    };
    let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TriangleStrip,
                                               &[1 as u16, 2, 0, 3]).unwrap();
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",
            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    vec4 color = texture(tex, v_tex_coords);
                    f_color = vec4(color.rgb * color.a, 1.0);
                }
            "
        },
    ).unwrap();

    let mut editor = Editor::new();
    editor.read_text(&String::from_utf8(std::fs::read("level.txt").unwrap()).unwrap());
    //editor.read_data_file(&std::fs::read("../level.bin").unwrap());
    //println!("{}", editor.write_text());
    let mut events = vec![];
    let data = unsafe {
        let fb8 = fb.as_ptr() as *const u8;
        std::slice::from_raw_parts(fb8, fb.len() * 4)
    };
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&data, (960, 540));
    let mut opengl_texture = glium::texture::SrgbTexture2d::with_mipmaps(&display, image, glium::texture::MipmapsOption::NoMipmap).unwrap();
    event_loop.run(move |ev, _, cfl| {
        use glium::glutin::{event::*, event_loop};
        let process = match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *cfl = event_loop::ControlFlow::Exit;
                    return;
                },
                c => { events.push(c.to_static().unwrap()); return; }
            },
            Event::MainEventsCleared => true,
            Event::RedrawRequested(..) => false,
            _ => return
        };
        let data = unsafe {
            let fb8 = fb.as_ptr() as *const u8;
            std::slice::from_raw_parts(fb8, fb.len() * 4)
        };
        if editor.process_frame(&mut fb, &mut events) {
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&data, (960, 540));
            opengl_texture.write(glium::Rect { left: 0, bottom: 0, width: 960, height: 540 }, image);
        }
        let mut target = display.draw();

        let uniforms = glium::uniform! {
            tex: opengl_texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    })
}

struct LevelTerrain {
    loc: u8,
    data: Vec<u8>,
    decoded_len: usize,
}

struct Editor {
    level_land: Vec<u8>,
    level_terrain: Vec<LevelTerrain>,
    foreground: foreground::Foreground,
    last_mouse: Vec2<i32>,
    dragging: bool,
    camera: Vec2<i32>,
    rerender: bool
}

impl Editor {
    fn new() -> Self {
        let mut foreground = foreground::Foreground::new();
        Self {
            level_land: vec![],
            level_terrain: vec![],
            foreground,
            camera: vec2(0, 0),
            last_mouse: vec2(0, 0),
            dragging: false,
            rerender: true
        }
    }
    fn read_text(&mut self, src: &str) -> Result<(), String> {
        let mut level_land = vec![];
        let mut iter = src.lines();
        for i in iter.by_ref() {
            if i.is_empty() { break; }
            let ch = i.chars().take_while(|&c| c != '#').filter(|c| c.is_ascii_hexdigit()).collect::<String>();
            let l = (0..ch.len()).step_by(2).filter_map(|i| {
                u8::from_str_radix(ch.get(i..i+2)?, 16).ok()
            }).collect::<Vec<u8>>();
            if let &[x,y,w,h,b] = &l[..] {
                level_land.push(x);
                level_land.push(y);
                let param_w = if w > 0x0F { 0 } else { w };
                let param_h = if h > 0x0F { 0 } else { h };
                level_land.push((param_w << 4) | param_h);
                if param_w == 0 { level_land.push(w); }
                if param_h == 0 { level_land.push(h); }
                level_land.push(b);
            } else {
                return Err("weird land def".into());
            }
            println!("land: {:02X?}", l);
        }
        self.level_land = level_land;
        terrain::decode_land(self.foreground.blocks_mut(), &self.level_land);

        self.level_terrain.clear();
        let mut loc = 0x00;
        let mut offset = 0;
        for i in iter {
            if i.starts_with("loc ") {
                loc = u8::from_str_radix(&i[4..], 16).map_err(|c| format!("failed parsing: {}", c))?;
                println!("loc {:02X}", loc);
                let x = loc as usize >> 4;
                let y = loc as usize & 0x0F;
                offset = (16 * x as usize) + (16 * y as usize) * 256;
                continue;
            }
            let ch = i.chars().take_while(|&c| c != '#').filter(|c| c.is_ascii_hexdigit()).collect::<String>();
            if ch == "" { continue; }
            let chunk = (0..ch.len()).step_by(2).filter_map(|i| {
                u8::from_str_radix(ch.get(i..i+2)?, 16).ok()
            }).collect::<Vec<u8>>();
            let mut off = 1;
            terrain::decode_object(chunk[0], &mut self.foreground.blocks_mut()[offset..], &mut std::iter::from_fn(|| {
                let c = chunk.get(off).copied().unwrap_or(0);
                off += 1;
                Some(c)
            }));
            self.level_terrain.push(LevelTerrain {
                loc, data: chunk.to_vec(), decoded_len: off
            });
            println!("obj: {:02X?}", chunk);
        }
        Ok(())
    }
    fn write_text(&mut self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let mut land = &self.level_land[..];
        while land.len() != 0 {
            let data = terrain::decode_land_chunk(&mut land);
            for i in data.iter() {
                write!(out, "{:02X} ", i);
            }
            out.pop();
            writeln!(out);
        }
        writeln!(out);
        let mut cur_loc = 0;
        for i in self.level_terrain.iter() {
            if i.loc != cur_loc {
                writeln!(out, "loc {:02X}", i.loc);
                cur_loc = i.loc;
            }
            for i in i.data.iter() {
                write!(out, "{:02X} ", i);
            }
            out.pop();
            writeln!(out);
        }
        out
    }
    fn read_data_file(&mut self, src: &[u8]) {
        let len = ((src[0] as usize) << 8) + src[1] as usize;
        let src = &src[2..];
        self.level_land = src[..len].to_vec();
        terrain::decode_land(self.foreground.blocks_mut(), &self.level_land);
        self.level_terrain.clear();
        let mut src = &src[len..];
        while src.len() != 0 {
            let loc = src[0];
            let len = src[1] as usize;
            let x = loc as usize >> 4;
            let y = loc as usize & 0x0F;
            let chunk = &src[2..2+len];
            let offset = (16 * x as usize) + (16 * y as usize) * 256;
            let mut off = 0;
            while let Some(id) = chunk.get(off) {
                let old_off = off;
                off += 1;
                terrain::decode_object(*id, &mut self.foreground.blocks_mut()[offset..], &mut std::iter::from_fn(|| {
                    let c = chunk.get(off).copied();
                    off += 1;
                    c
                }));
                self.level_terrain.push(LevelTerrain {
                    loc, data: chunk[old_off..off].to_vec(), decoded_len: off - old_off
                });
                println!("obj: {:02X?}", &chunk[old_off..off]);
            }
            src = &src[2+len..];
        }
    }
    fn write_data_file(&mut self) -> Vec<u8> {
        let mut out = vec![];
        out.push((self.level_land.len() >> 8) as _);
        out.push((self.level_land.len() & 0xFF) as _);
        out.extend_from_slice(&self.level_land);
        let mut loc = 0;
        let mut chunk = vec![];
        for i in self.level_terrain.iter() {
            if i.loc != loc {
                out.push(loc);
                out.push(chunk.len() as u8);
                out.extend_from_slice(&chunk);
                loc = i.loc;
                chunk.clear();
            }
            for c in 0..i.decoded_len {
                chunk.push(i.data.get(c).copied().unwrap_or(0));
            }
        }
        out.push(loc);
        out.push(chunk.len() as u8);
        out.extend_from_slice(&chunk);
        out
    }
    fn process_frame(&mut self, fb: &mut [u32], events: &mut Vec<WindowEvent>) -> bool {
        use glium::glutin::event;
        let mut mouse_pos = self.last_mouse;
        for i in events.drain(..) {
            match i {
                WindowEvent::CursorMoved { position, .. } => {
                    let [x,y]: [f64;2] = position.into();
                    mouse_pos = vec2(x as i32,y as i32) / 2;
                    self.rerender = true;
                },
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == event::MouseButton::Middle {
                        self.dragging = state == event::ElementState::Pressed;
                    }
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    use glium::glutin::event::VirtualKeyCode;
                    if input.virtual_keycode == Some(VirtualKeyCode::R) {
                        for i in self.foreground.blocks_mut().iter_mut() {
                            *i = 0;
                        }
                        self.read_text(&String::from_utf8(std::fs::read("level.txt").unwrap()).unwrap());
                    }
                    if input.virtual_keycode == Some(VirtualKeyCode::W) {
                        std::fs::write("../level_demo.bin", self.write_data_file());
                    }
                    self.rerender = true;
                },
                _ => {}
            }
        }
        if self.dragging && self.last_mouse != mouse_pos {
            self.camera += self.last_mouse - mouse_pos;
            self.rerender = true;
        }
        let redraw_frame = self.rerender;
        self.last_mouse = mouse_pos;
        if self.rerender {
            self.render(fb);
            self.rerender = false;
        }
        redraw_frame
    }
    fn render(&mut self, fb: &mut [u32]) {
        let width = 960;
        for (i,c) in fb.iter_mut().enumerate() {
            let pos = vec2(i % width, i / width).map(|c| c as i32);
            let transp = (pos / 4) & 1;
            let transp = if transp.x ^ transp.y == 0 { 0xFF111111 } else { 0xFF222222 };
            let pos = pos + self.camera;
            if pos.x >= 0 && pos.y >= 0 && pos.x < 4096 && pos.y < 4096 {
                if pos.x % 256 == 0 || pos.y % 256 == 0 {
                    *c = 0xFF808080;
                } else {
                    *c = self.foreground.sample_pixel(pos).filter(|&c| c > 0x1000000).unwrap_or(transp);
                }
            } else {
                *c = transp;
            }
        }
        let mut fb = Framebuffer {
            fb, width
        };
        /*let mut iter = self.level_land.iter();
        while iter.as_slice().len() != 0 {
            let x = *iter.next().unwrap() as i32;
            let y = *iter.next().unwrap() as i32;
            let params = *iter.next().unwrap() as i32;
            let mut h = params & 0x0F;
            let mut w = params >> 4;
            if w == 0 { w = *iter.next().unwrap() as i32; }
            if h == 0 { h = *iter.next().unwrap() as i32; }
            let x = x * 16;
            let y = y * 16;
            let w = w * 16;
            let h = h * 16;
            let b = *iter.next().unwrap();
            for i in x..x+w {
                fb.pixel(vec2(i, y  ) - self.camera).map(|c| *c = 0xFF008000);
                fb.pixel(vec2(i, y+h) - self.camera).map(|c| *c = 0xFF008000);
            }
            for i in y..y+h {
                fb.pixel(vec2(x,   i) - self.camera).map(|c| *c = 0xFF008000);
                fb.pixel(vec2(x+w, i) - self.camera).map(|c| *c = 0xFF008000);
            }
        }*/
        for i in self.level_terrain.iter() {
            let y_offset = (i.loc & 0x0F) as i32;
            let x_offset = (i.loc >> 4) as i32;
            let y_page = (i.data[1] & 0x0F) as i32;
            let x_page = (i.data[1] >> 4) as i32;
            let mut pos = vec2(x_offset * 256 + x_page * 16, y_offset * 256 + y_page * 16) - self.camera;
            graphics::draw_text(&mut fb, &mut pos, format!("{:02X}", i.data[0]).as_bytes());
        }
        let mut pos = self.last_mouse + vec2(0,10);
        let block = (self.last_mouse + self.camera) / 16;
        if block.map(|c| c >= 0 && c < 256) == vec2(true,true) {
            let x = block.x * 16;
            let y = block.y * 16;
            let w = 16; let h = 16;
            for i in x..x+w {
                fb.pixel(vec2(i, y  ) - self.camera).map(|c| *c = 0xFFFFFFFF);
                fb.pixel(vec2(i, y+h) - self.camera).map(|c| *c = 0xFFFFFFFF);
            }
            for i in y..y+h {
                fb.pixel(vec2(x,   i) - self.camera).map(|c| *c = 0xFFFFFFFF);
                fb.pixel(vec2(x+w, i) - self.camera).map(|c| *c = 0xFFFFFFFF);
            }
            graphics::draw_text(&mut fb, &mut pos, format!("{:02X},{:02X}", block.x, block.y).as_bytes());
        }
    }
}

pub struct Framebuffer<'a> {
    fb: &'a mut [u32],
    width: usize
}
impl FbSurface for Framebuffer<'_> {
    fn pixel(&mut self, pos: Vec2<i32>) -> Option<&mut u32> {
        let c = pos.x > 0 && pos.x < self.width as i32;
        self.fb.get_mut(pos.x as usize + pos.y as usize * self.width).filter(|_| c)
    }
}
