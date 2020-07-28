use tooth_engine::{*, framebuffer::Framebuffer};

use glium::program;
use glium::Surface;

pub static mut BUF: Framebuffer = Framebuffer::new();

fn main() {
    let fb = unsafe { &mut BUF };
    let mut state = unsafe { state::get() };
    /*for i in 0..100 {
        state.run(fb, buttons);
    }*/
    println!("{}", std::mem::size_of_val(state));
    let mut event_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(320.0*4.0, 180.0*4.0))
        .with_title("WASM TAS tools");
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
    let mut buttons = controller::Buttons { current: 0, old: 0 };
    let mut keys_pressed = std::collections::HashSet::new();
    let mut current_frame = 0;
    event_loop.run(move |ev, _, cfl| {
        use glium::glutin::{event::*, event_loop};
        let process = match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *cfl = event_loop::ControlFlow::Exit;
                    return;
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    use glium::glutin::event::ElementState;
                    input.virtual_keycode.map(|c| if input.state == ElementState::Pressed {
                        keys_pressed.insert(c);
                    } else {
                        keys_pressed.remove(&c);
                    });
                    return;
                },
                c => { /*events.push(c);*/ return; }
            },
            Event::MainEventsCleared => true,
            Event::RedrawRequested(..) => false,
            _ => return
        };
        if process {
            use glium::glutin::event::VirtualKeyCode::*;
            buttons.old = buttons.current;
            buttons.current = 0;
            if keys_pressed.contains(&Left)  { buttons.current |= 0x01; }
            if keys_pressed.contains(&Right) { buttons.current |= 0x02; }
            if keys_pressed.contains(&Up)    { buttons.current |= 0x04; }
            if keys_pressed.contains(&Down)  { buttons.current |= 0x08; }
            if keys_pressed.contains(&Space) { buttons.current |= 0x10; }
            if keys_pressed.contains(&Z)     { buttons.current |= 0x20; }
            if keys_pressed.contains(&X)     { buttons.current |= 0x40; }
            if keys_pressed.contains(&C)     { buttons.current |= 0x80; }
            state.run(fb, buttons);
        }
        let data = unsafe {
            let fb8 = fb.as_ptr() as *const u8;
            std::slice::from_raw_parts(fb8, fb.len() * 4)
        };
        /*image::ImageBuffer::<image::Rgba<u8>,_>::from_raw(320, 180, data).map(|c| {
            c.save(&format!("frame{}.png", current_frame));
        });*/
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&data, (320, 180));
        let opengl_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
        let mut target = display.draw();

        let uniforms = glium::uniform! {
            tex: opengl_texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
        current_frame += 1;
    })
}
