pub struct RandState {
    state: i32,
}

impl RandState {
    pub const fn new(state: i32) -> Self {
        Self { state }
    }
    // XorShift LFSR
    pub fn next(&mut self) -> i32 {
        self.state += 1;
        self.state ^= self.state >> 6;
        self.state ^= self.state << 12;
        self.state ^= self.state >> 13;
        self.state
    }
}
/*
impl Iterator for RandState {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        Some(self.next())
    }
}*/
