//! Piece abstraction. This module does not specify the actual shapes for the pieces or
//! their wall kicks.

use core::mem::transmute;
use core::ops::Range;

use crate::matrix::{Mat, MatBuf};

/// Represents the position of a shape. This includes the rotation state.
pub struct Pos {
    pub x: i8,
    pub y: i8,
    pub r: Rot,
}

impl From<(i8, i8)> for Pos {
    fn from((x, y): (i8, i8)) -> Self {
        let r = Rot::default();
        Pos { x, y, r }
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

impl Rot {
    #[inline]
    pub fn cw(self) -> Self {
        u8::from(self).wrapping_add(1).into()
    }

    #[inline]
    pub fn ccw(self) -> Self {
        u8::from(self).wrapping_add(3).into()
    }
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

/// Represents the state of a piece (its shape and position).
pub struct Piece<T> {
    pub shape: T,
    pub pos: Pos,
}

/// Abstraction for determining spawn locations.
pub trait Spawn {
    fn spawn(&self) -> (i8, i8);
}

impl<T: Spawn> Piece<T> {
    pub fn spawn(shape: T) -> Self {
        let pos = shape.spawn().into();
        Self { shape, pos }
    }
}

/// Abstraction for determining the cells of a shape.
pub trait Shape {
    fn cells(&self, r: Rot) -> Cells;
}

/// Represents the occupied cells of a piece. This data is computed from a [`Shape`] at a
/// specific [`Rot`] and is primarily used to check for collision with a [`Mat`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Cells {
    x0: i8,
    x1: i8,
    y0: i8,
    y1: i8,
    bits: u16,
}

impl Cells {
    /// The boundary extents are represented by the ranges `xs` and `ys`, and the cell
    /// data is represented by `bits`.
    pub const fn new(xs: Range<i8>, ys: Range<i8>, bits: u16) -> Self {
        assert!(xs.end >= xs.start);
        assert!(ys.end >= ys.start);
        Self {
            bits,
            x0: xs.start,
            x1: xs.end,
            y0: ys.start,
            y1: ys.end,
        }
    }

    pub const fn offset(&self, x: i8, y: i8) -> Self {
        Self {
            bits: self.bits,
            /* wrapping is technically buggy but i'm going to cross my fingers and hope
             * that it doesn't occur for real 10x20 boards. */
            x0: self.x0.wrapping_add(x),
            x1: self.x1.wrapping_add(x),
            y0: self.y0.wrapping_add(y),
            y1: self.y1.wrapping_add(y),
        }
    }

    pub fn collides(&self, mat: &Mat) -> bool {
        if self.x0 < 0 || self.x1 >= mat.cols() || self.y0 < 0 {
            return true;
        }

        let y0 = self.y0;
        let y1 = self.y1.min(mat.len());
        let x0 = self.x0;
        let mut bits = self.bits;
        let mut test = 0;

        for y in y0..y1 {
            let mask = (bits & 0b1111) << x0;
            test |= unsafe { mat.get_unchecked(y) } & mask;
            bits >>= 4;
        }

        test != 0
    }

    pub fn immobile(&self, mat: &Mat) -> bool {
        self.offset(0, -1).collides(mat)
            || self.offset(0, 1).collides(mat)
            || self.offset(-1, 0).collides(mat)
            || self.offset(1, 0).collides(mat)
    }

    pub fn place(&self, mat: &mut MatBuf) {
        let y0 = self.y0;
        let y1 = self.y1;
        let x0 = self.x0;
        let mut bits = self.bits;

        for y in y0..y1 {
            let mask = (bits & 0b1111) << x0;
            mat.set(y, mask);
            bits >>= 4;
        }
    }
}

impl<T: Shape> Piece<T> {
    /// Get the cells occupied by the piece.
    pub fn cells(&self) -> Cells {
        self.shape.cells(self.pos.r).offset(self.pos.x, self.pos.y)
    }
}
