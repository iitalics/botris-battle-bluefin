use alloc::vec::Vec;

use crate::matrix::Mat;
use crate::piece::{Dir, Piece, Pos, Shape, Turn};

type HashBuilder = core::hash::BuildHasherDefault<ahash::AHasher>;
type HashSet<T> = hashbrown::HashSet<T, HashBuilder>;

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

    // spawn the initial piece to start the search
    let mut pc = Piece::spawn(piece_type);

    // if piece is above the top row, immediately drop it to right above the matrix. this
    // hopefully avoids some checks when computing sonic drop
    let (_, ys) = pc.cells().extents();
    let y0 = ys.start;
    let y_top = matrix.len();
    if y0 > y_top {
        pc.pos.y -= y0 - y_top;
    }

    if !pc.cells().collides(matrix) {
        // if this is not true, then we are dead
        places.push(pc.pos);
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

type PlacesResult<T> = Piece<T>;
// pub struct PlacesResult<T> {
//     pub piece: Piece<T>,
//     pub is_immobile: bool,
// }

impl<T> Iterator for Places<'_, T>
where
    T: Shape + Clone,
{
    type Item = PlacesResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut pc = self.pop()?;

            let mut pc_cw = pc.clone();
            if pc_cw.try_rotate(self.matrix, Turn::Cw).is_some() {
                self.push(pc_cw.pos);
            }

            let mut pc_ccw = pc.clone();
            if pc_ccw.try_rotate(self.matrix, Turn::Ccw).is_some() {
                self.push(pc_ccw.pos);
            }

            let mut pc_l = pc.clone();
            if pc_l.try_shift(self.matrix, Dir::Left).is_some() {
                self.push(pc_l.pos);
            }

            let mut pc_r = pc.clone();
            if pc_r.try_shift(self.matrix, Dir::Right).is_some() {
                self.push(pc_r.pos);
            }

            let (dy, _) = pc.sonic_drop(self.matrix);
            if dy != 0 {
                // piece was floating so we don't return it yet; push it on the stack and
                // return it when reached in a future iteration
                self.push(pc.pos);
                continue;
            }

            return Some(pc);
        }
    }
}
