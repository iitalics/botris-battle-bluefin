use core::mem::transmute;
use core::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(i8)]
pub enum Input {
    Left = 0,
    Right = 2,
    Cw = 1,
    Ccw = 3,
    // Drop = 4,
    SonicDrop = 5,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(i8)]
pub enum Dir {
    Left = -1,
    Right = 1,
}

impl From<Dir> for i8 {
    fn from(r: Dir) -> Self {
        r as i8
    }
}

impl ops::Add<Dir> for i8 {
    type Output = i8;
    fn add(self, rhs: Dir) -> i8 {
        self + rhs as i8
    }
}

impl From<Dir> for Input {
    fn from(dx: Dir) -> Self {
        match dx {
            Dir::Left => Input::Left,
            Dir::Right => Input::Right,
        }
    }
}

/// Represents the rotation state of a shape.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[repr(u8)]
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
    fn from(v: u8) -> Self {
        unsafe { transmute(v & 3) }
    }
}

impl From<Rot> for u8 {
    fn from(r: Rot) -> Self {
        r as u8
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i8)]
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

impl From<Turn> for Input {
    fn from(dr: Turn) -> Self {
        match dr {
            Turn::Cw => Input::Cw,
            Turn::Ccw => Input::Ccw,
        }
    }
}
