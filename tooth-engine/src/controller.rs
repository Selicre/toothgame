
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Buttons {
    pub current: u32,
    pub old: u32
}

macro_rules! buttons {
    ($($val:expr, $const:ident, $func:ident, $func_edge:ident;)*) => {
        impl Buttons {
            pub fn has(self, pressed: u32) -> bool {
                self.current & pressed != 0
            }
            pub fn has_edge(self, pressed: u32) -> bool {
                (self.current & !self.old) & pressed != 0
            }
            $(
                pub const $const: u32 = 1<<$val;
                pub fn $func(self) -> bool { self.has(Self::$const) }
                pub fn $func_edge(self) -> bool { self.has_edge(Self::$const) }
            )*
        }
    }
}

buttons! {
    0, LEFT, left, left_edge;
    1, RIGHT, right, right_edge;
    2, UP, up, up_edge;
    3, DOWN, down, down_edge;
    4, START, start, start_edge;
    5, A, a, a_edge;
    6, B, b, b_edge;
    7, C, c, c_edge;
}
