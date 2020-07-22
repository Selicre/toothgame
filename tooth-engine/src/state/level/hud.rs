use super::LevelState;
use crate::framebuffer::Framebuffer;
use crate::vec2::{vec2, Vec2};
use crate::graphics::{self, DUNE_FG, BOLDFACE};

pub struct Hud {
    textbox: Textbox,
    position: Vec2<i32>
}

impl Hud {
    pub fn new() -> Self {
        Self {
            textbox: Textbox::empty(),
            position: vec2(16, 8)
        }
    }
    pub fn render(&mut self, fb: &mut Framebuffer, parent: *mut LevelState) {
        project!(parent.{data, entity_set});
        let mut position = self.position;
        graphics::draw_text(fb, &mut position, b"SCORE     COINS  TIME\n");
        let mut buf = [b' '; 15];
        dec_format(data.score, &mut buf[..8], true);
        dec_format(data.coins, &mut buf[10..], false);
        graphics::draw_text(fb, &mut position, &buf);
        if let Some(ref mut timer) = &mut data.timer {
            if *timer < 10*60*60 - 1 { *timer += 1; }
            position += vec2(20, 0);
            let minutes = *timer / 60 / 60;
            let seconds = *timer / 60 % 60;
            buf[0] = minutes as u8 + b'0';
            buf[1] = b':';
            graphics::draw_text(fb, &mut position, &buf[..2]);
            dec_format(seconds, &mut buf[..2], true);
            position -= vec2(4, 0);
            graphics::draw_text(fb, &mut position, &buf[..2]);
        }
        if cfg!(feature = "debug") {
            position.x = 16;
            position.y += 16;
            let mut target = *b"POS 000000 000000";

            let pos = entity_set.player.pos();
            hex_format(pos.x, &mut target[4..10], false);
            hex_format(pos.y, &mut target[11..17], false);

            graphics::draw_text(fb, &mut position, &target[..]);
        }
        self.textbox.render(fb);
    }
    #[inline(always)]   // to make sure the const fn gets inlined
    pub fn show_textbox(&mut self, msg: &'static [u8]) {
        self.textbox = Textbox::parse_str(msg);
    }
    pub fn hide_textbox(&mut self) {
        self.textbox.msg = None;
    }
}

pub struct Textbox {
    msg: Option<&'static [u8]>,
    timer: i32,
    height: i32,
    bounds: Vec2<i32>
}

impl Textbox {
    pub const fn empty() -> Self {
        Self {
            msg: None,
            timer: 0,
            height: 0,
            bounds: vec2(0,0)
        }
    }
    pub const fn parse_str(s: &'static [u8]) -> Self {
        Self {
            msg: Some(s),
            timer: 0,
            height: 0,
            bounds: get_bounds(s)
        }
    }
    pub fn render(&mut self, fb: &mut Framebuffer) {
        let hlen = self.bounds.x;
        let vlen = self.bounds.y;

        let hstart = (Framebuffer::WIDTH as i32 / 2) - (hlen as i32 * 4) - 8;
        let hend = (Framebuffer::WIDTH as i32 / 2) + (hlen as i32 * 4) + 8;
        if self.height > 0 {
            let vstart = (36 - self.height).max(24);
            let vend = 36 + self.height;
            for i in hstart..hend {
                for j in vstart..vend {
                    *fb.pixel(vec2(i,j)).unwrap() = if i == hstart || i == hend-1 || j == vstart || j == vend-1 {
                        //0xFF83212c
                        DUNE_FG.get_pal()[5]
                    } else if ((i + self.timer / 2) / 16) % 2 != ((j + self.timer / 2) / 16) % 2 {
                        //0xFFcc2b32
                        DUNE_FG.get_pal()[2]
                    } else {
                        //0xFFd04a61
                        DUNE_FG.get_pal()[3]
                    };
                }
            }
            self.timer += 1;
        }
        if let Some(msg) = self.msg {
            let mut position = vec2(hstart + 8, 32);
            graphics::draw_text(fb, &mut position, &msg[..(self.timer as usize).min(msg.len())]);
            if self.height < 4 + vlen as i32 * 8 {
                self.height += 1;
            }
        } else {
            if self.height > 0 {
                self.height -= 1;
            }
        }
    }
}

fn dec_format(mut num: i32, target: &mut [u8], zero_pad: bool) {
    for i in (0..target.len()).rev() {
        target[i] = (num % 10) as u8 + b'0';
        num /= 10;
        if !zero_pad && num == 0 { break; }
    }
}

fn hex_format(mut num: i32, target: &mut [u8], zero_pad: bool) {
    for i in (0..target.len()).rev() {
        let ch = (num & 15) as u8;
        target[i] = ch + if ch > 9 {
            b'A' - 10
        } else {
            b'0'
        };
        num >>= 4;
        if !zero_pad && num == 0 { break; }
    }
}

const fn get_bounds(s: &[u8]) -> Vec2<i32> {
    let mut idx = 0;
    let mut count = 0;
    let mut width = 0;
    let mut line = 0;
    loop {
        line += 1;
        if line > width {
            width = line;
        }
        if s[idx] == b'\n' {
            count += 1;
            line = 0;
        }
        idx += 1;
        if idx == s.len() { break; }
    }
    vec2(width, count + 1)
}
