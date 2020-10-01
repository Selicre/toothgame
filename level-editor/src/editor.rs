
use tooth_engine::*;

use framebuffer::{Surface as FbSurface};
use crate::vec2::{Vec2,vec2};

use glium::program;
use glium::Surface;
use glium::glutin::event::WindowEvent;

use glium::glutin::event::VirtualKeyCode as KeyCode;

use std::collections::HashSet;

struct LevelLand {
    payload: [u8; 5]
}

struct LevelTerrain {
    payload: Vec<u8>,
    decoded_len: usize,
}
impl LevelTerrain {
    pub fn id(&self) -> u8 {
        self.payload[0]
    }
    pub fn y(&self) -> u8 {
        self.payload[1]
    }
    pub fn x(&self) -> u8 {
        self.payload[2]
    }
    pub fn params(&self) -> u8 {
        self.payload[3]
    }
    pub fn id_mut(&mut self) -> &mut u8 {
        &mut self.payload[0]
    }
    pub fn y_mut(&mut self) -> &mut u8 {
        &mut self.payload[1]
    }
    pub fn x_mut(&mut self) -> &mut u8 {
        &mut self.payload[2]
    }
    pub fn params_mut(&mut self) -> &mut u8 {
        &mut self.payload[3]
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct GrabbedObj {
    id: usize,
    corners: usize,
    offset: Vec2<i32>
}

pub enum EditorMode {
    Land {
        grabbed_land: Option<GrabbedObj>,
        selected_land: Vec<usize>,
    },
    Objects {
        grabbed_object: Option<GrabbedObj>
    },
    Preview
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Land {
            grabbed_land: None,
            selected_land: vec![]
        }
    }
}

#[derive(Default)]
pub struct EditorState {
    grabbing: bool,
    releasing: bool,
    multiselect: bool,
    mode: EditorMode
}

pub struct Editor {
    level_land: Vec<LevelLand>,
    level_terrain: Vec<LevelTerrain>,
    foreground: foreground::Foreground,
    mouse_pos: Vec2<i32>,
    last_mouse: Vec2<i32>,
    dragging: bool,
    camera: Vec2<i32>,
    rerender: bool,
    rebuild: bool,
    keys: HashSet<KeyCode>,

    state: EditorState
}

impl Editor {
    pub fn new() -> Self {
        let mut foreground = foreground::Foreground::new();
        Self {
            level_land: vec![],
            level_terrain: vec![],
            foreground,
            camera: vec2(0, 0),
            mouse_pos: vec2(0, 0),
            last_mouse: vec2(0, 0),
            dragging: false,
            rerender: true,
            rebuild: true,
            keys: Default::default(),

            state: EditorState::default()
        }
    }
    pub fn process_frame(&mut self, fb: &mut [u32], events: &mut Vec<WindowEvent>) -> bool {
        use glium::glutin::event;
        for i in events.drain(..) {
            match i {
                WindowEvent::CursorMoved { position, .. } => {
                    let [x,y]: [f64;2] = position.into();
                    self.mouse_pos = vec2(x as i32,y as i32) / 2;
                    self.rerender = true;
                },
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == event::MouseButton::Middle {
                        self.dragging = state == event::ElementState::Pressed;
                    } else if button == event::MouseButton::Left {
                        self.state.grabbing = state == event::ElementState::Pressed;
                        self.state.releasing = state != event::ElementState::Pressed;
                        self.rerender = true;
                    }
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    use KeyCode::*;
                    let pressed = input.state == event::ElementState::Pressed;
                    if let Some(c) = input.virtual_keycode { match c {
                        LShift => self.state.multiselect = pressed,
                        /*R => {
                            for i in self.foreground.blocks_mut().iter_mut() {
                                *i = 0;
                            }
                            //self.read_text(&String::from_utf8(std::fs::read("level.txt").unwrap()).unwrap());
                        }*/
                        W if pressed => { std::fs::write("../level_demo.bin", self.write_level().unwrap()); }
                        L if pressed => self.state.mode = EditorMode::Land {
                            grabbed_land: None,
                            selected_land: vec![]
                        },
                        O if pressed => self.state.mode = EditorMode::Objects {
                            grabbed_object: None,
                        },
                        P if pressed => self.state.mode = EditorMode::Preview,
                        _ => {},
                    } if pressed { self.keys.insert(c); } }
                    self.rerender = true;
                },
                _ => {}
            }
        }
        if self.dragging && self.last_mouse != self.mouse_pos {
            self.camera += self.last_mouse - self.mouse_pos;
            self.rerender = true;
        }
        let redraw_frame = self.rerender;
        self.last_mouse = self.mouse_pos;
        if self.rerender {
            let mut fb = Framebuffer {
                fb, width: 960
            };
            self.render(&mut fb);
            self.rerender = false;
        }
        if self.rebuild {
            self.rebuild();
            self.rebuild = false;
            self.rerender = true;
        }
        self.keys.clear();
        self.state.grabbing = false;
        self.state.releasing = false;
        redraw_frame
    }
    fn edit_objects(&mut self, fb: &mut Framebuffer) {
        if let EditorMode::Objects { ref mut grabbed_object } = &mut self.state.mode {
            let world_pos = self.mouse_pos + self.camera;
            if self.state.releasing {
                println!("releasing");
                *grabbed_object = None;
            }
            if self.keys.contains(&KeyCode::A) {
                let wp = (world_pos / 16).map(|c| c as u8);
                self.level_terrain.push(LevelTerrain {
                    payload: vec![0x01, wp.y, wp.x, 0x22],
                    decoded_len: 4,
                });
                self.rebuild = true;
            }
            let mut action_id = None;
            for (id, obj) in self.level_terrain.iter_mut().enumerate() {
                if let Some(c) = grabbed_object.as_ref() {
                    if c.id == id {
                        *obj.x_mut() = (world_pos.x / 16) as u8;
                        *obj.y_mut() = (world_pos.y / 16) as u8;
                        self.rebuild = true;
                    }
                }
                let pos = vec2(obj.x() as i32, obj.y() as i32) * 16;
                let x_range = (pos.x..pos.x + 16).contains(&world_pos.x);
                let y_range = (pos.y..pos.y + 16).contains(&world_pos.y);
                if self.state.grabbing {
                    println!("trying to grab {}", id);
                    if x_range && y_range {
                        println!("grabbing");
                        *grabbed_object = Some(GrabbedObj { id, corners: 15, offset: vec2(0,0) });
                    }
                }
                if x_range && y_range {
                    action_id = Some((id, world_pos - pos));
                }

                let mut pos = vec2(obj.x() as i32, obj.y() as i32) * 16 - self.camera;
                graphics::draw_text(fb, &mut pos, format!("{:02X}\n{:02X}", obj.id(), obj.params()).as_bytes());
            }
            if let Some((id, offset)) = action_id {
                if self.keys.contains(&KeyCode::Delete) {
                    self.level_terrain.remove(id);
                    self.rebuild = true;
                } else if self.keys.contains(&KeyCode::PageUp) {
                    if id != 0 {
                        self.level_terrain.swap(id, id-1);
                        self.rebuild = true;
                    }
                } else if self.keys.contains(&KeyCode::PageDown) {
                    if id != self.level_terrain.len()-1 {
                        self.level_terrain.swap(id, id+1);
                        self.rebuild = true;
                    }
                } else if self.keys.contains(&KeyCode::Z) {
                    let quadrant = if offset.x >= 8 { 1 } else { 0 }
                                 | if offset.y >= 8 { 2 } else { 0 };
                    let item = &mut self.level_terrain[id];
                    match quadrant {
                        0 => *item.id_mut() = item.id().wrapping_sub(16),
                        1 => *item.id_mut() = item.id().wrapping_sub(1),
                        2 => *item.params_mut() = item.params().wrapping_sub(16),
                        3 => *item.params_mut() = (item.params() & 0xF0) | (item.params().wrapping_sub(1) & 0x0F),
                        _ => {}
                    }
                    self.rebuild = true;
                } else if self.keys.contains(&KeyCode::X) {
                    let quadrant = if offset.x >= 8 { 1 } else { 0 }
                                 | if offset.y >= 8 { 2 } else { 0 };
                    let item = &mut self.level_terrain[id];
                    match quadrant {
                        0 => *item.id_mut() = item.id().wrapping_add(16),
                        1 => *item.id_mut() = item.id().wrapping_add(1),
                        2 => *item.params_mut() = item.params().wrapping_add(16),
                        3 => *item.params_mut() = (item.params() & 0xF0) | (item.params().wrapping_add(1) & 0x0F),
                        _ => {}
                    }
                    self.rebuild = true;
                }
            }
            graphics::draw_text(fb, &mut vec2(8, 540-16), format!("EDITING OBJECTS").as_bytes());
        }
    }
    fn edit_land(&mut self, fb: &mut Framebuffer) {
        if let EditorMode::Land { ref mut grabbed_land, ref mut selected_land } = &mut self.state.mode {
            let mut iter = &self.level_land[..];
            if self.state.releasing {
                println!("releasing");
                *grabbed_land = None;
            }
            if self.state.grabbing && self.state.multiselect {
                selected_land.clear();
            }
            let world_pos = self.mouse_pos + self.camera;

            let mut delete_id = None;
            if self.keys.contains(&KeyCode::A) {
                let wp = (world_pos / 16).map(|c| c as u8);
                self.level_land.push(LevelLand {
                    payload: [wp.x, wp.y, 3, 3, 0x62]
                });
                self.rebuild = true;
            }
            if self.keys.contains(&KeyCode::S) {
                let wp = (world_pos / 16).map(|c| c as u8);
                self.level_land.push(LevelLand {
                    payload: [wp.x, wp.y, 3, 3, 0x00]
                });
                self.rebuild = true;
            }
            for (id,payload) in self.level_land.iter_mut().enumerate().map(|(i,c)| (i, &mut c.payload)) {
                let [ref mut x,ref mut y,ref mut w,ref mut h,ref mut b] = payload;
                let mut grabbed = false;
                if let Some(c) = grabbed_land.as_ref() {
                    if c.id == id {
                        grabbed = true;
                        let world_pos = self.mouse_pos + self.camera;
                        let offset = (world_pos / 16).map(|c| c as u8);
                        let offset2 = ((world_pos - c.offset) / 16).map(|c| c as u8);
                        if c.corners == 15 {
                            *x = offset2.x;
                            *y = offset2.y;
                        } else {
                            if c.corners & 2 != 0 {
                                *w = offset.x.saturating_sub(*x).max(1);
                            }
                            if c.corners & 8 != 0 {
                                *h = offset.y.saturating_sub(*y).max(1);
                            }
                        }
                        self.rebuild = true;
                    }
                }

                let x = *x as i32 * 16;
                let y = *y as i32 * 16;
                let w = *w as i32 * 16;
                let h = *h as i32 * 16;
                let x_range = (x-4..x+w+4).contains(&world_pos.x);
                let y_range = (y-4..y+h+4).contains(&world_pos.y);
                if self.state.grabbing {
                    println!("trying to grab..");
                    let mut grab = 0;
                    if (world_pos.x - x    ).abs() < 4 && y_range { grab |= 1; }
                    if (world_pos.x - (x+w)).abs() < 4 && y_range { grab |= 2; }
                    if (world_pos.y - y    ).abs() < 4 && x_range { grab |= 4; }
                    if (world_pos.y - (y+h)).abs() < 4 && x_range { grab |= 8; }
                    if grab == 0 && x_range && y_range {
                        grab = 15;
                    }
                    if grab != 0 {
                        let offset = world_pos - vec2(x,y);
                        *grabbed_land = Some(GrabbedObj { id, corners: grab, offset });
                        grabbed = true;
                    }
                }
                if x_range && y_range && self.keys.contains(&KeyCode::Delete) {
                    delete_id = Some(id);
                }
                let color = if grabbed {
                    0xFF008080
                } else {
                    0xFF008000
                };
                for i in x..x+w {
                    fb.pixel(vec2(i, y  ) - self.camera).map(|c| *c = color);
                    fb.pixel(vec2(i, y+h) - self.camera).map(|c| *c = color);
                }
                for i in y..y+h {
                    fb.pixel(vec2(x,   i) - self.camera).map(|c| *c = color);
                    fb.pixel(vec2(x+w, i) - self.camera).map(|c| *c = color);
                }
            }
            if let Some(id) = delete_id {
                self.level_land.remove(id);
                self.rebuild = true;
            }
            graphics::draw_text(fb, &mut vec2(8, 540-16), format!("EDITING LAND").as_bytes());
        }
    }
    fn render(&mut self, fb: &mut Framebuffer) {
        let width = 960;
        for (i,c) in fb.fb.iter_mut().enumerate() {
            let pos = vec2(i % width, i / width).map(|c| c as i32);
            let transp = (pos / 4) & 1;
            let transp = if transp.x ^ transp.y == 0 { 0xFF111111 } else { 0xFF222222 };
            let pos = pos + self.camera;
            if pos.x >= 0 && pos.y >= 0 && pos.x < 4096 && pos.y < 4096 {
                if (pos.x % 256 == 0 || pos.y % 256 == 0) && !matches!(self.state.mode, EditorMode::Preview) {
                    *c = 0xFF808080;
                } else {
                    *c = self.foreground.sample_pixel(pos).filter(|&c| c > 0x1000000).unwrap_or(transp);
                }
            } else {
                *c = transp;
            }
        }
        self.edit_land(fb);
        self.edit_objects(fb);
        let mut pos = self.mouse_pos + vec2(0,10);
        let block = (self.mouse_pos + self.camera) / 16;
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
            graphics::draw_text(fb, &mut pos, format!("{:02X},{:02X}", block.x, block.y).as_bytes());
        }
    }
    pub fn read_level(&mut self, src: &[u8]) -> Result<(), String> {
        self.clear_blocks();
        let len = ((src[0] as usize) << 8) + src[1] as usize;
        let src = &src[2..];
        self.level_land.clear();
        {
            let mut src = &src[..len];
            terrain::decode_land(self.foreground.blocks_mut(), &src);
            while src.len() != 0 {
                self.level_land.push(LevelLand { payload: terrain::decode_land_chunk(&mut src) });
            }
        }
        let mut src = &src[len..];
        self.level_terrain.clear();
        while src.len() != 0 {
            let old = &src[..];
            terrain::decode_object(self.foreground.blocks_mut(), &mut src);
            let len = src.as_ptr() as usize - old.as_ptr() as usize;
            self.level_terrain.push(LevelTerrain {
                payload: old[..len].to_vec(),
                decoded_len: len
            });
        }
        Ok(())
    }
    pub fn write_level(&mut self) -> Result<Vec<u8>, String> {
        let mut buf = vec![];
        buf.extend_from_slice(&((self.level_land.len() * 5) as u16).to_be_bytes());
        for i in self.level_land.iter() {
            buf.extend_from_slice(&i.payload);
        }
        for i in self.level_terrain.iter() {
            buf.extend_from_slice(&i.payload[..i.decoded_len]);
        }
        Ok(buf)
    }
    pub fn clear_blocks(&mut self) {
        for i in self.foreground.blocks_mut().iter_mut() {
            *i = 0;
        }
    }
    pub fn rebuild(&mut self) {
        self.clear_blocks();
        terrain::decode_land_with(
            self.foreground.blocks_mut(),
            self.level_land.iter().map(|c| c.payload)
        );
        for i in self.level_terrain.iter_mut() {
            let mut buf = [0; 256];
            buf[..i.payload.len()].copy_from_slice(&i.payload);
            let mut src = &buf[..];
            terrain::decode_object(self.foreground.blocks_mut(), &mut src);
            i.decoded_len = buf.len() - src.len();
        }
    }
    /*
    pub fn read_text(&mut self, src: &str) -> Result<(), String> {
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
                let y = loc as usize >> 4;
                let x = loc as usize & 0x0F;
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
    pub fn write_text(&mut self) -> String {
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
    pub fn rebuild(&mut self) {
        for i in self.foreground.blocks_mut().iter_mut() {
            *i = 0;
        }
        terrain::decode_land(self.foreground.blocks_mut(), &self.level_land);
        for i in self.level_terrain.iter_mut() {
            let y = (i.loc >> 4) as usize;
            let x = (i.loc & 0x0F) as usize;
            let offset = x * 16 + y * 256 * 16;

            let mut off = 1;
            terrain::decode_object(i.data[0], &mut self.foreground.blocks_mut()[offset..], &mut std::iter::from_fn(|| {
                let c = i.data.get(off).copied();
                off += 1;
                Some(c.unwrap_or(0))
            }));
            i.decoded_len = off;
        }
    }
    pub fn read_data_file(&mut self, src: &[u8]) {
        let len = ((src[0] as usize) << 8) + src[1] as usize;
        let src = &src[2..];
        self.level_land = src[..len].to_vec();
        terrain::decode_land(self.foreground.blocks_mut(), &self.level_land);
        self.level_terrain.clear();
        let mut src = &src[len..];
        while src.len() != 0 {
            let loc = src[0];
            let len = src[1] as usize;
            let y = loc as usize >> 4;
            let x = loc as usize & 0x0F;
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
    pub fn write_data_file(&mut self) -> Vec<u8> {
        // transform land
        let mut l = vec![];
        let mut iter = &self.level_land[..];
        while iter.len() > 0 {
            let s = terrain::decode_land_chunk(&mut iter);
            l.extend_from_slice(&s);
        }

        let mut out = vec![];
        out.push((l.len() >> 8) as _);
        out.push((l.len() & 0xFF) as _);
        out.extend_from_slice(&l);
        let mut loc = 0;
        //let mut chunk = vec![];
        for i in self.level_terrain.iter() {
            /*if i.loc != loc {
                out.push(loc);
                out.push(chunk.len() as u8);
                out.extend_from_slice(&chunk);
                loc = i.loc;
                chunk.clear();
            }
            for c in 0..i.decoded_len {
                chunk.push(i.data.get(c).copied().unwrap_or(0));
            }*/
            let mut data = i.data.clone();
            let x = data[1] >> 4;
            let y = data[1] & 0xF;
            let loc_y = loc >> 4;
            let loc_x = loc & 0xF;
            data[1] = y + (loc_y << 4);
            data.insert(2, x + (loc_x << 4));
            out.append(&mut data);
        }
        //out.push(loc);
        //out.push(chunk.len() as u8);
        //out.extend_from_slice(&chunk);
        out
    }*/
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
