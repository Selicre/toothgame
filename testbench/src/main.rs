use tooth_engine::{*, framebuffer::Framebuffer};

pub static mut BUF: Framebuffer = Framebuffer::new();

fn main() {
    unsafe {
        let fb = &mut BUF;
        let buttons = controller::Buttons(0);
        let mut state = state::get();
        state.run(fb, buttons);
        state.run(fb, buttons);
        state.run(fb, buttons);
        state.run(fb, buttons);
        state.run(fb, buttons);
        println!("{}", std::mem::size_of_val(state));
        let fb8 = fb.as_ptr() as *const u8;
        let fb8 = std::slice::from_raw_parts(fb8, fb.len() * 4);
        image::ImageBuffer::<image::Rgba<_>, _>::from_raw(320, 180, fb8).unwrap().save("test.png");
    }
}
