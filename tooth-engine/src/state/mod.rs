//! Tracking the game's state.

use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::graphics;

pub mod level;

pub static mut STATE: Option<GameState> = None;

pub enum GameState {
    Level(level::State)
}

impl GameState {
    pub fn new() -> Self {
        graphics::init();
        GameState::Level(level::State::new())
    }
    pub fn run(&mut self, fb: &mut Framebuffer, buttons: Buttons) {
        let out = match self {
            GameState::Level(st) => st.run(fb, buttons)
        };
        if let Some(out) = out {
            *self = out;
        }
    }
}

// SAFETY: don't call this twice.
pub unsafe fn get() -> &'static mut GameState {
    STATE.get_or_insert(GameState::new())
}
