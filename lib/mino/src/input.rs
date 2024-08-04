use core::mem::transmute;
use core::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Dir {
    Left = -1,
    Right = 1,
}

impl ops::Add<Dir> for i8 {
    type Output = i8;
    fn add(self, rhs: Dir) -> i8 {
        self + rhs as i8
    }
}

/// Represents the rotation state of a shape.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub enum Rot {
    /// Initial orientation.
    #[default]
    N = 0,
    /// One CW rotation.
    E = 1,
    /// Two rotations in either direction.
    S = 2,
    /// One CCW rotation, or three CW rotations.
    W = 3,
}

impl From<u8> for Rot {
    #[inline]
    fn from(v: u8) -> Self {
        unsafe { transmute(v & 3) }
    }
}

impl From<Rot> for u8 {
    #[inline]
    fn from(r: Rot) -> Self {
        r as u8
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Turn {
    Cw = 1,
    Ccw = 3,
}

impl ops::Add<Turn> for Rot {
    type Output = Rot;
    fn add(self, t: Turn) -> Self::Output {
        (self as u8 + t as u8).into()
    }
}
