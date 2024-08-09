//! Piece abstraction. This module does not specify the actual shapes for the pieces or
//! their wall kicks.

use core::fmt;
use core::ops::Range;
use core::str;

use crate::input::{Dir, Rot, Turn};
use crate::matrix::{Mat, MatBuf};

/// Represents the position of a shape. This includes the rotation state.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl From<(i8, i8, Rot)> for Pos {
    fn from((x, y, r): (i8, i8, Rot)) -> Self {
        Pos { x, y, r }
    }
}

impl PartialEq<(i8, i8, Rot)> for Pos {
    fn eq(&self, other: &(i8, i8, Rot)) -> bool {
        *self == Pos::from(*other)
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.x, self.y, self.r).fmt(f)
    }
}

/// Represents the state of a piece (its shape and position).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Piece<T> {
    pub shape: T,
    pub pos: Pos,
}

impl<T> Piece<T> {
    pub fn new(shape: T, pos: impl Into<Pos>) -> Self {
        Self {
            shape,
            pos: pos.into(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Piece<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Piece")
            .field(&self.shape)
            .field(&self.pos.x)
            .field(&self.pos.y)
            .field(&self.pos.r)
            .finish()
    }
}

/// Interface for pieces that can be spawned.
pub trait Spawn {
    fn spawn(&self) -> (i8, i8);
}

impl<T: Spawn> Piece<T> {
    pub fn spawn(shape: T) -> Self {
        let pos = shape.spawn();
        Self::new(shape, pos)
    }
}

/// Interface for pieces that have a list of wall kicks.
pub trait WallKicks {
    fn wall_kicks(&self, r: Rot, dr: Turn) -> &'static [(i8, i8)];
}

/// Interface for pieces that have [`Cells`] representations of their shape.
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

    /// Offset the cells by the given amount.
    pub fn offset(&self, x: i8, y: i8) -> Self {
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

    /// Returns the boundary extents of the cells, representing the range of cells taken
    /// up along the x and y axis respectively.
    pub fn extents(&self) -> (Range<i8>, Range<i8>) {
        (self.x0..self.x1, self.y0..self.y1)
    }

    pub fn bottom(&self) -> i8 {
        self.y0
    }

    /// Returns the list of coordinates occupied by the cells. This is not particularly
    /// efficient so should be used only for tests.
    pub fn coords(&self) -> impl Iterator<Item = (i8, i8)> {
        let x0 = self.x0;
        let y0 = self.y0;
        let bits = self.bits;
        (0..16i8)
            .filter(move |i| bits & (1 << i) != 0)
            .map(move |i| {
                let x = i % 4;
                let y = i / 4;
                (x0 + x, y0 + y)
            })
    }

    /// Returns `true` if any of the cells collides with the given matrix.
    pub fn collides(&self, mat: &Mat) -> bool {
        if self.x0 < 0 || self.x1 > mat.cols() || self.y0 < 0 {
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

    /// Returns `true` if the cells are immobile.
    pub fn immobile(&self, mat: &Mat) -> bool {
        self.offset(0, -1).collides(mat)
            && self.offset(0, 1).collides(mat)
            && self.offset(-1, 0).collides(mat)
            && self.offset(1, 0).collides(mat)
    }

    /// Places the cells onto the matrix by filling in the occupied coordinates.
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

impl fmt::Debug for Cells {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugCoords {
            w: usize,
            h: usize,
            bits: u16,
        }

        impl fmt::Debug for DebugCoords {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut f = f.debug_list();
                for y in 0..self.h {
                    let bs: [u8; 4] = core::array::from_fn(|x| {
                        if self.bits & (1 << (y * 4 + x)) != 0 {
                            b'x'
                        } else {
                            b'.'
                        }
                    });
                    let s = str::from_utf8(&bs[..self.w]).unwrap();
                    f.entry(&s);
                }
                f.finish()
            }
        }

        let (xs, ys) = self.extents();
        let coords = DebugCoords {
            w: xs.len(),
            h: ys.len(),
            bits: self.bits,
        };

        f.debug_tuple("Cells")
            .field(&xs)
            .field(&ys)
            .field(&coords)
            .finish()
    }
}

impl<T: Shape> Piece<T> {
    /// Get the cells occupied by the piece.
    pub fn cells(&self) -> Cells {
        self.shape.cells(self.pos.r).offset(self.pos.x, self.pos.y)
    }

    /// Try to move in the given direction. If there is no collision, returns
    /// `Some(final_cells)` and shifts the piece. If there is a collision, returns `None`
    /// and leaves the piece unmodified.
    pub fn try_shift(&mut self, mat: &Mat, dx: Dir) -> Option<Cells> {
        let r = self.pos.r;
        let x = self.pos.x + dx;
        let y = self.pos.y;
        let cells = self.shape.cells(r).offset(x, y);

        if !cells.collides(mat) {
            self.pos.x = x;
            return Some(cells);
        }

        None
    }

    /// Try to rotate in the given direction. If there is no collision (applying the wall
    /// kicks), returns `Some(final_cells)` and rotates the piece. If there is a
    /// collision, returns `None` and leaves the piece unmodified.
    pub fn try_rotate(&mut self, mat: &Mat, dr: Turn) -> Option<Cells>
    where
        T: WallKicks,
    {
        let r0 = self.pos.r;
        let r = self.pos.r + dr;
        let x = self.pos.x;
        let y = self.pos.y;
        let cells = self.shape.cells(r).offset(x, y);

        for &(dx, dy) in self.shape.wall_kicks(r0, dr) {
            let cells = cells.offset(dx, dy);
            if !cells.collides(mat) {
                self.pos.x += dx;
                self.pos.y += dy;
                self.pos.r = r;
                return Some(cells);
            }
        }

        None
    }

    /// Sonic drop the piece so that it touches the stack. Returns the vertical distance
    /// fell, and the final cells. If the distance is zero then the piece is already on
    /// the stack.
    pub fn sonic_drop(&mut self, mat: &Mat) -> (i8, Cells) {
        let mut dy = 0;
        let cells = self.cells();

        // immediately drop to the top of the matrix if the piece is above it
        let y0 = cells.bottom();
        let y_top = mat.len();
        if y0 > y_top {
            dy = y_top - y0;
        }

        while !cells.offset(0, dy - 1).collides(mat) {
            dy -= 1;
        }

        self.pos.y += dy;
        (-dy, cells.offset(0, dy))
    }
}
