
use tooth_engine::*;

use framebuffer::{Surface as FbSurface};
use crate::vec2::{Vec2,vec2};

use glium::program;
use glium::Surface;
use glium::glutin::event::WindowEvent;

mod editor;
use editor::Editor;

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
    //editor.read_text(&String::from_utf8(std::fs::read("level.txt").unwrap()).unwrap());
    editor.read_level(&std::fs::read("../level_demo.bin").unwrap());
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

