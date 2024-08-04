use alloc::vec::Vec;

use crate::matrix::Mat;
use crate::piece::{Dir, Piece, Pos, Shape, Spawn, Turn, WallKicks};

type HashSet<T> = hashbrown::HashSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

pub struct Places<'m, T> {
    matrix: &'m Mat,
    piece_type: T,
    stack: Vec<Pos>,
    visited: HashSet<Pos>,
}

pub fn places<'m, T>(matrix: &'m Mat, piece_type: T) -> Places<'m, T>
where
    T: Shape + Spawn + Clone,
{
    let mut places = Places {
        matrix,
        piece_type: piece_type.clone(),
        stack: Vec::with_capacity(64),
        visited: HashSet::with_capacity(256),
    };

    let spawn_piece = Piece::spawn(piece_type);
    if !spawn_piece.cells().collides(matrix) {
        // if this is not true, then we are dead
        places.push(spawn_piece.pos);
    }

    places
}

impl<T> Places<'_, T>
where
    T: Shape + Clone,
{
    fn push(&mut self, pos: Pos) -> bool {
        if !self.visited.insert(pos) {
            return false;
        }
        self.stack.push(pos);
        true
    }

    fn pop(&mut self) -> Option<Piece<T>> {
        Some(Piece::new(self.piece_type.clone(), self.stack.pop()?))
    }
}

pub struct PlacesResult<T> {
    pub piece: Piece<T>,
    pub is_immobile: bool,
}

impl<T> Iterator for Places<'_, T>
where
    T: Shape + WallKicks + Clone,
{
    type Item = PlacesResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut piece = self.pop()?;

            let mut cw = piece.clone();
            if cw.try_rotate(self.matrix, Turn::Cw).is_some() {
                self.push(cw.pos);
            }

            let mut ccw = piece.clone();
            if ccw.try_rotate(self.matrix, Turn::Ccw).is_some() {
                self.push(ccw.pos);
            }

            let mut left = piece.clone();
            while left.try_shift(self.matrix, Dir::Left).is_some() {
                if !self.push(left.pos) {
                    break;
                }
            }

            let mut right = piece.clone();
            while right.try_shift(self.matrix, Dir::Right).is_some() {
                if !self.push(right.pos) {
                    break;
                }
            }

            let (dy, cells) = piece.sonic_drop(self.matrix);
            if dy != 0 {
                // piece was floating so we don't return it yet; push it on the stack and
                // return it when reached in a future iteration
                self.push(piece.pos);
                continue;
            }

            let is_immobile = cells.immobile(self.matrix);
            return Some(PlacesResult { piece, is_immobile });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::matrix::MatBuf;
    use crate::piece::Rot;
    use crate::standard_rules::PieceType;
    use crate::test::assert_same_set;
    use core::ops::RangeInclusive;

    fn assert_places(
        piece: PieceType,
        mat: &Mat,
        expected: impl IntoIterator<Item = (RangeInclusive<i8>, i8, Rot)>,
        immobile: impl IntoIterator<Item = (i8, i8, Rot)>,
    ) {
        let immobile = immobile.into_iter().map(Pos::from).collect::<Vec<_>>();
        let actual_places = places(mat, piece).map(|res| {
            let pos = res.piece.pos;
            assert_eq!(res.is_immobile, immobile.contains(&pos), "{pos:?}");
            pos
        });
        let expected_places = expected
            .into_iter()
            .flat_map(|(xs, y, r)| xs.map(move |x| (x, y, r).into()));
        assert_same_set(actual_places, expected_places, &piece);
    }

    #[test]
    fn test_t_places() {
        assert_places(
            PieceType::T,
            Mat::empty(),
            [
                (0..=7, 1, Rot::N),
                (-1..=7, 2, Rot::E),
                (0..=7, 2, Rot::S),
                (0..=8, 2, Rot::W),
            ],
            [],
        );
    }

    #[test]
    fn test_i_places() {
        assert_places(
            PieceType::I,
            Mat::empty(),
            [
                (0..=6, 1, Rot::N),
                (0..=6, 2, Rot::S), // same as N
                (-2..=7, 3, Rot::E),
                (-1..=8, 3, Rot::W), // same as W
            ],
            [],
        );
    }

    #[test]
    fn test_s_spin_places() {
        let mut mat = MatBuf::new();
        // 1 xxxxx..xxx
        // 0 xxxx..xxxx
        //   0123456789
        mat.set(0, 0b1111001111);
        mat.set(1, 0b1110011111);
        //
        assert_places(
            PieceType::S,
            &mat,
            [
                (0..=4, 3, Rot::N),
                (5..=5, 2, Rot::N),
                (6..=7, 3, Rot::N),
                (0..=4, 4, Rot::S),
                (5..=5, 3, Rot::S),
                (6..=7, 4, Rot::S),
                (-1..=2, 4, Rot::E),
                (3..=4, 3, Rot::E),
                (5..=7, 4, Rot::E),
                (0..=3, 4, Rot::W),
                (4..=5, 3, Rot::W),
                (6..=8, 4, Rot::W),
                // spin
                (4..=4, 2, Rot::S),
            ],
            [(4, 2, Rot::S)],
        );
    }
}
