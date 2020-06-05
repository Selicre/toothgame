use core::ops;

pub const fn vec2<T>(x: T, y: T) -> Vec2<T> { Vec2 { x, y } }

#[derive(Copy,Clone,Default,Debug,PartialEq,Eq,Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Vec2<U> {
        Vec2 {
            x: f(self.x),
            y: f(self.y)
        }
    }
}

impl<T> From<[T;2]> for Vec2<T> {
    fn from([x,y]: [T;2]) -> Self {
        Self { x, y }
    }
}

impl<T: ops::Add> ops::Add for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn add(self, other: Self) -> Self::Output {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}
/*
impl<T: ops::Add> ops::AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        *self = self + other;
    }
}*/

impl<T: ops::Sub> ops::Sub for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn sub(self, other: Self) -> Self::Output {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl<T: ops::Mul + Clone> ops::Mul<T> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn mul(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x * other.clone(),
            y: self.y * other
        }
    }
}

impl<T: ops::Div + Clone> ops::Div<T> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn div(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x / other.clone(),
            y: self.y / other
        }
    }
}

impl<T: ops::Rem + Clone> ops::Rem<T> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn rem(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x % other.clone(),
            y: self.y % other
        }
    }
}

impl<T: ops::BitAnd + Clone> ops::BitAnd<T> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn bitand(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x & other.clone(),
            y: self.y & other
        }
    }
}

impl<T: ops::BitOr + Clone> ops::BitOr<T> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn bitor(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x | other.clone(),
            y: self.y | other
        }
    }
}
