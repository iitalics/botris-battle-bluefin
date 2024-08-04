use alloc::vec::Vec;

use crate::matrix::Mat;
use crate::piece::{Dir, Piece, Pos, Shape, Turn};

type HashSet<T> = hashbrown::HashSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

pub struct Places<'m, T> {
    matrix: &'m Mat,
    piece_type: T,
    stack: Vec<Pos>,
    visited: HashSet<Pos>,
}

pub fn places<'m, T>(matrix: &'m Mat, piece_type: T) -> Places<'m, T>
where
    T: Shape + Clone,
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
    T: Shape + Clone,
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
            if left.try_shift(self.matrix, Dir::Left).is_some() {
                self.push(left.pos);
            }

            let mut right = piece.clone();
            if right.try_shift(self.matrix, Dir::Right).is_some() {
                self.push(right.pos);
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
    use crate::piece::Rot::*;
    use crate::standard_rules::PieceType;
    use crate::test::assert_same_set;

    #[test]
    fn test_t_places() {
        let piece = PieceType::T;
        let mat = Mat::empty();
        assert_same_set(
            places(mat, piece).map(|r| {
                assert!(!r.is_immobile);
                r.piece.cells()
            }),
            [
                (0, 1, N),
                (1, 1, N),
                (2, 1, N),
                (3, 1, N),
                (4, 1, N),
                (5, 1, N),
                (6, 1, N),
                (7, 1, N),
                (-1, 2, E),
                (0, 2, E),
                (1, 2, E),
                (2, 2, E),
                (3, 2, E),
                (4, 2, E),
                (5, 2, E),
                (6, 2, E),
                (7, 2, E),
                (0, 2, S),
                (1, 2, S),
                (2, 2, S),
                (3, 2, S),
                (4, 2, S),
                (5, 2, S),
                (6, 2, S),
                (7, 2, S),
                (0, 2, W),
                (1, 2, W),
                (2, 2, W),
                (3, 2, W),
                (4, 2, W),
                (5, 2, W),
                (6, 2, W),
                (7, 2, W),
                (8, 2, W),
            ]
            .map(|pos| Piece::new(piece, pos).cells()),
            "places({piece})",
        );
    }

    #[test]
    fn test_i_places() {
        let piece = PieceType::I;
        let mat = Mat::empty();
        assert_same_set(
            places(mat, piece).map(|r| {
                assert!(!r.is_immobile);
                r.piece.cells()
            }),
            [
                (0, 1, N),
                (1, 1, N),
                (2, 1, N),
                (3, 1, N),
                (4, 1, N),
                (5, 1, N),
                (6, 1, N),
                (-2, 3, E),
                (-1, 3, E),
                (0, 3, E),
                (1, 3, E),
                (2, 3, E),
                (3, 3, E),
                (4, 3, E),
                (5, 3, E),
                (6, 3, E),
                (7, 3, E),
            ]
            .map(|pos| Piece::new(piece, pos).cells()),
            "places({piece})",
        );
    }
}
