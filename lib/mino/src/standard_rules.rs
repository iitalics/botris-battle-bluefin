//! Standard implementation of tetris pieces.

use core::fmt;

use super::piece::{Cells, Rot, Turn};
use super::piece::{Shape, Spawn, WallKicks};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum PieceType {
    I = 0,
    J = 1,
    L = 2,
    O = 3,
    S = 4,
    T = 5,
    Z = 6,
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: PieceType::name()
        <Self as fmt::Debug>::fmt(self, f)
    }
}

pub type Piece = super::piece::Piece<PieceType>;

// w=4
// ----------
// .......... 20
// ...IIII... 19
// 0123456789

// w=3
// ----------
// .....L.... 20
// ...LLL.... 19
// 0123456789

// w=2
// ----------
// ....OO.... 20
// ....OO.... 19
// 0123456789

const SPAWN_2: (i8, i8) = (10 / 2 - 2 / 2, 20);
const SPAWN_3_4: (i8, i8) = (10 / 2 - 4 / 2, 20);

impl Spawn for PieceType {
    fn spawn(&self) -> (i8, i8) {
        if *self == PieceType::O {
            SPAWN_2
        } else {
            SPAWN_3_4
        }
    }
}

static CELLS: [[Cells; 4]; 7] = [
    // .... ..I. .... .I..
    // IIII ..I. .... .I..
    // .... ..I. IIII .I..
    // .... ..I. .... .I..
    [
        Cells::new(0..4, -1..0, 0b_1111),
        Cells::new(2..3, -3..1, 0b_0001_0001_0001_0001),
        Cells::new(0..4, -2..-1, 0b_1111),
        Cells::new(1..2, -3..1, 0b_0001_0001_0001_0001),
    ],
    // J.. .JJ ... .J.
    // JJJ .J. JJJ .J.
    // ... .J. ..J JJ.
    [
        Cells::new(0..3, -1..1, 0b_0001_0111),
        Cells::new(1..3, -2..1, 0b_0011_0001_0001),
        Cells::new(0..3, -2..0, 0b_0111_0100),
        Cells::new(0..2, -2..1, 0b_0010_0010_0011),
    ],
    // ..L .L. ... LL.
    // LLL .L. LLL .L.
    // ... .LL L.. .L.
    [
        Cells::new(0..3, -1..1, 0b_0100_0111),
        Cells::new(1..3, -2..1, 0b_0001_0001_0011),
        Cells::new(0..3, -2..0, 0b_0111_0001),
        Cells::new(0..2, -2..1, 0b_0011_0010_0010),
    ],
    // OO
    // OO
    [
        Cells::new(0..2, -1..1, 0b_0011_0011),
        Cells::new(0..2, -1..1, 0b_0011_0011),
        Cells::new(0..2, -1..1, 0b_0011_0011),
        Cells::new(0..2, -1..1, 0b_0011_0011),
    ],
    // .SS .S. ... S..
    // SS. .SS .SS SS.
    // ... ..S SS. .S.
    [
        Cells::new(0..3, -1..1, 0b_0110_0011),
        Cells::new(1..3, -2..1, 0b_0001_0011_0010),
        Cells::new(0..3, -2..0, 0b_0110_0011),
        Cells::new(0..2, -2..1, 0b_0001_0011_0010),
    ],
    // .T. .T. ... .T.
    // TTT .TT TTT TT.
    // ... .T. .T. .T.
    [
        Cells::new(0..3, -1..1, 0b_0010_0111),
        Cells::new(1..3, -2..1, 0b_0001_0011_0001),
        Cells::new(0..3, -2..0, 0b_0111_0010),
        Cells::new(0..2, -2..1, 0b_0010_0011_0010),
    ],
    // ZZ. ..Z ... .Z.
    // .ZZ .ZZ ZZ. ZZ.
    // ... .Z. .ZZ Z..
    [
        Cells::new(0..3, -1..1, 0b_0011_0110),
        Cells::new(1..3, -2..1, 0b_0010_0011_0001),
        Cells::new(0..3, -2..0, 0b_0011_0110),
        Cells::new(0..2, -2..1, 0b_0010_0011_0001),
    ],
];

impl Shape for PieceType {
    fn cells(&self, r: Rot) -> Cells {
        CELLS[*self as usize][r as usize]
    }
}

static WALLKICKS: [[[(i8, i8); 5]; 2]; 4] = [
    [
        /* 0-1 */ [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        /* 0-3 */ [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    ],
    [
        /* 1-2 */ [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        /* 1-0 */ [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    ],
    [
        /* 2-3 */ [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        /* 2-1 */ [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    ],
    [
        /* 3-0 */ [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        /* 3-2 */ [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    ],
];

static I_WALLKICKS: [[[(i8, i8); 5]; 2]; 4] = [
    [
        /* 0-1 */ [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        /* 0-3 */ [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    ],
    [
        /* 1-2 */ [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        /* 1-0 */ [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    ],
    [
        /* 2-3 */ [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        /* 2-1 */ [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    ],
    [
        /* 3-0 */ [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        /* 3-2 */ [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    ],
];

impl WallKicks for PieceType {
    fn wall_kicks(&self, r: Rot, dr: Turn) -> &'static [(i8, i8)] {
        let i = r as usize;
        let j = (dr as usize) >> 1; // Cw => 0, Ccw => 1
        if *self == PieceType::I {
            &I_WALLKICKS[i][j]
        } else {
            &WALLKICKS[i][j]
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::matrix::Mat;
    use crate::piece::Pos;
    use crate::test::assert_same_set;
    use std::format;

    fn cells(p: PieceType, x: i8, y: i8, r: Rot) -> Cells {
        Piece {
            shape: p,
            pos: (x, y, r).into(),
        }
        .cells()
    }

    fn assert_cells(p: PieceType, x: i8, y: i8, r: Rot, coords: [(i8, i8); 4]) {
        let piece = Piece {
            shape: p,
            pos: (x, y, r).into(),
        };
        let cells = piece.cells();
        assert_same_set(cells.coords(), coords, &format!("{piece:?}"));

        let (mut x0, mut y0, mut x1, mut y1) = (i8::MAX, i8::MAX, i8::MIN, i8::MIN);
        for (cx, cy) in coords {
            x0 = x0.min(cx);
            y0 = y0.min(cy);
            x1 = x1.max(cx + 1);
            y1 = y1.max(cy + 1);
        }
        assert_eq!(cells.extents(), (x0..x1, y0..y1), "{piece:?}");
    }

    #[test]
    fn test_cells() {
        use PieceType::*;
        assert_cells(I, 3, 20, Rot::N, [(3, 19), (4, 19), (5, 19), (6, 19)]);
        assert_cells(I, 3, 20, Rot::E, [(5, 20), (5, 19), (5, 18), (5, 17)]);
        assert_cells(I, 3, 20, Rot::S, [(3, 18), (4, 18), (5, 18), (6, 18)]);
        assert_cells(I, 3, 20, Rot::W, [(4, 20), (4, 19), (4, 18), (4, 17)]);
        assert_cells(J, 3, 20, Rot::N, [(3, 19), (4, 19), (5, 19), (3, 20)]);
        assert_cells(J, 3, 20, Rot::E, [(4, 18), (4, 19), (4, 20), (5, 20)]);
        assert_cells(J, 3, 20, Rot::S, [(3, 19), (4, 19), (5, 19), (5, 18)]);
        assert_cells(J, 3, 20, Rot::W, [(4, 18), (4, 19), (4, 20), (3, 18)]);
        assert_cells(L, 3, 20, Rot::N, [(3, 19), (4, 19), (5, 19), (5, 20)]);
        assert_cells(L, 3, 20, Rot::E, [(4, 18), (4, 19), (4, 20), (5, 18)]);
        assert_cells(L, 3, 20, Rot::S, [(3, 19), (4, 19), (5, 19), (3, 18)]);
        assert_cells(L, 3, 20, Rot::W, [(4, 18), (4, 19), (4, 20), (3, 20)]);
        assert_cells(O, 4, 20, Rot::N, [(4, 19), (5, 19), (4, 20), (5, 20)]);
        assert_cells(O, 4, 20, Rot::E, [(4, 19), (5, 19), (4, 20), (5, 20)]);
        assert_cells(O, 4, 20, Rot::S, [(4, 19), (5, 19), (4, 20), (5, 20)]);
        assert_cells(O, 4, 20, Rot::W, [(4, 19), (5, 19), (4, 20), (5, 20)]);
        assert_cells(S, 3, 20, Rot::N, [(3, 19), (4, 19), (4, 20), (5, 20)]);
        assert_cells(S, 3, 20, Rot::E, [(4, 19), (4, 20), (5, 18), (5, 19)]);
        assert_cells(S, 3, 20, Rot::S, [(3, 18), (4, 18), (4, 19), (5, 19)]);
        assert_cells(S, 3, 20, Rot::W, [(3, 19), (3, 20), (4, 18), (4, 19)]);
        assert_eq!(cells(S, 3, 20, Rot::N), cells(S, 3, 21, Rot::S));
        assert_eq!(cells(S, 3, 20, Rot::E), cells(S, 4, 20, Rot::W));
        assert_cells(T, 3, 20, Rot::N, [(3, 19), (4, 19), (5, 19), (4, 20)]);
        assert_cells(T, 3, 20, Rot::E, [(4, 18), (4, 19), (4, 20), (5, 19)]);
        assert_cells(T, 3, 20, Rot::S, [(3, 19), (4, 19), (5, 19), (4, 18)]);
        assert_cells(T, 3, 20, Rot::W, [(4, 18), (4, 19), (4, 20), (3, 19)]);
        assert_cells(Z, 3, 20, Rot::N, [(3, 20), (4, 20), (4, 19), (5, 19)]);
        assert_cells(Z, 3, 20, Rot::E, [(4, 19), (4, 18), (5, 20), (5, 19)]);
        assert_cells(Z, 3, 20, Rot::S, [(3, 19), (4, 19), (4, 18), (5, 18)]);
        assert_cells(Z, 3, 20, Rot::W, [(3, 19), (3, 18), (4, 20), (4, 19)]);
        assert_eq!(cells(Z, 3, 20, Rot::N), cells(Z, 3, 21, Rot::S));
        assert_eq!(cells(Z, 3, 20, Rot::E), cells(Z, 4, 20, Rot::W));
    }

    #[test]
    fn test_spawn() {
        assert_eq!(Piece::spawn(PieceType::I).pos, Pos::from((3, 20, Rot::N)));
        assert_eq!(Piece::spawn(PieceType::J).pos, Pos::from((3, 20, Rot::N)));
        assert_eq!(Piece::spawn(PieceType::L).pos, Pos::from((3, 20, Rot::N)));
        assert_eq!(Piece::spawn(PieceType::O).pos, Pos::from((4, 20, Rot::N)));
        assert_eq!(Piece::spawn(PieceType::S).pos, Pos::from((3, 20, Rot::N)));
        assert_eq!(Piece::spawn(PieceType::T).pos, Pos::from((3, 20, Rot::N)));
        assert_eq!(Piece::spawn(PieceType::Z).pos, Pos::from((3, 20, Rot::N)));
    }

    #[test]
    fn test_wall_kick_lookup() {
        assert_eq!(
            PieceType::Z.wall_kicks(Rot::N, Turn::Cw),
            [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)]
        );
        assert_eq!(
            PieceType::J.wall_kicks(Rot::S, Turn::Ccw),
            [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        );
        assert_eq!(
            PieceType::I.wall_kicks(Rot::E, Turn::Ccw),
            [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        );
    }

    #[test]
    fn test_sonic_drop() {
        let mat = Mat::empty();
        let mut pc = Piece::spawn(PieceType::T);
        assert_eq!(pc.pos, (3, 20, Rot::N));
        let (dy, cells) = pc.sonic_drop(mat);
        assert_eq!(dy, 19);
        assert_eq!(pc.pos, (3, 1, Rot::N));
        assert_eq!(pc.cells(), cells);
        let (dy, cells) = pc.sonic_drop(mat);
        assert_eq!(dy, 0);
        assert_eq!(pc.pos, (3, 1, Rot::N));
        assert_eq!(pc.cells(), cells);
    }

    #[test]
    fn test_t_wall_kick() {
        let mat = Mat::empty();
        let mut pc = Piece::new(PieceType::T, (3, 1, Rot::N));
        let cells = pc.try_rotate(mat, Turn::Cw).expect("turn(Cw)");
        assert_eq!(pc.cells(), cells);
        assert_eq!(pc.pos, (2, 2, Rot::E));
    }
}
