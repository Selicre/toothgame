use core::ops;
use crate::vec2::{vec2, Vec2};

#[no_mangle]
pub static mut BUF: Framebuffer = Framebuffer::new();

pub struct Framebuffer {
    inner: [u32; Self::WIDTH * Self::HEIGHT]
}

impl Framebuffer {
    pub const WIDTH: usize = 320;
    pub const HEIGHT: usize = 180;

    pub const fn new() -> Self {
        Framebuffer {
            inner: [0; Self::WIDTH * Self::HEIGHT]
        }
    }

    pub fn pixels(&mut self) -> impl Iterator<Item=(Vec2<i32>, &mut u32)> {
        self.iter_mut().enumerate().map(|(i,c)| {
            let pos = vec2(i % Self::WIDTH, i / Self::WIDTH)
                .map(|c| c as i32);
            (pos, c)
        })
    }
}

impl ops::Deref for Framebuffer {
    type Target = [u32; Self::WIDTH * Self::HEIGHT];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for Framebuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
