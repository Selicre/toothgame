#![no_std]

use tooth_engine::{*, framebuffer::Framebuffer};

#[panic_handler]
unsafe fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    // Where we're going, we don't need safety.
    core::hint::unreachable_unchecked()
}

#[no_mangle]
pub static mut BUF: Framebuffer = Framebuffer::new();

#[no_mangle]
pub unsafe fn drw(buttons: u32) {
    let fb = &mut BUF;
    let buttons = controller::Buttons(buttons);
    state::get().run(fb, buttons);
}


#[no_mangle]
pub static mut SND: [f32; 1024] = [0.0; 1024];

#[no_mangle]
pub unsafe fn snd() {

}
