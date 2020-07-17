//! Tracking the game's state.

use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::graphics;

pub mod level;

pub static mut STATE: Option<GameState> = None;

pub enum GameState {
    Level(level::LevelState),
    Oops
}

impl GameState {
    pub fn new() -> Self {
        graphics::init();
        GameState::Level(level::LevelState::new())
    }
    pub fn run(&mut self, fb: &mut Framebuffer, buttons: Buttons) {
        let out = match self {
            GameState::Level(st) => st.run(fb, buttons),
            _ => None
        };
        if let Some(out) = out {
            *self = out;
        }
    }
    pub fn unwrap_level(&mut self) -> &mut level::LevelState {
        if let GameState::Level(ref mut l) = self {
            l
        } else {
            panic!()
        }
    }
}

// SAFETY: don't call this twice.
pub unsafe fn get() -> &'static mut GameState {
    STATE.get_or_insert_with(GameState::new)
}
